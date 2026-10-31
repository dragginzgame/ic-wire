//! Actual Wire-frame compression ratios and selected IC body-instruction samples.
#[expect(dead_code, reason = "Shared corpus has other measurement operations.")]
mod fixtures;
mod compression_transport;
use ic_platform::{candid::contract::Principal, pocket_ic::PocketIcBuilder};
use ic_testkit::pic::PocketIcStartupConfig;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, time::Duration};
fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args().nth(1).unwrap();
    let mut corpus = Vec::new();
    for case in fixtures::cases() {
        for lane in case.lanes {
            if lane.name.starts_with("wire-") { corpus.push((format!("{}/{}",case.name,lane.name),lane.bytes)); }
        }
    }
    for size in [0, 32, 2048, 65536, 1048576] {
        let mut state = 0x123456789abcdef_u64;
        let bytes: Vec<u8> = (0..size).map(|_| {state ^= state << 13; state ^= state >> 7; state ^= state << 17; state as u8}).collect();
        corpus.push((format!("random-{size}"),bytes));
        corpus.push((format!("zeros-{size}"),vec![0;size]));
    }
    let mut ratios = fs::File::create(format!("{out}/ratios.jsonl"))?;
    for (name, bytes) in &corpus {
        let compressed = compression_transport::compress(bytes);
        assert_eq!(compression_transport::decompress(&compressed).unwrap(), *bytes);
        writeln!(ratios,"{}",json!({"case":name,"sha256":hash(bytes),"raw_bytes":bytes.len(),"lz4_bytes":compressed.len()}))?;
    }
    assert!(compression_transport::decompress(&[]).is_err());
    assert!(compression_transport::decompress(&u32::MAX.to_le_bytes()).is_err());
    let mut invalid = compression_transport::compress(&[7;32]);
    invalid.pop(); assert!(compression_transport::decompress(&invalid).is_err());
    let mut invalid = compression_transport::compress(&[7;32]);
    invalid[..4].copy_from_slice(&33_u32.to_le_bytes());
    assert!(compression_transport::decompress(&invalid).is_err());
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))?
        .build_instance(PocketIcBuilder::new().with_application_subnet())?;
    let pic=owned.instance();
    let mut samples=fs::File::create(format!("{out}/samples.jsonl"))?;
    let mut caller=0_u64;
    // Eight evenly spaced real corpus frames plus all small/random/large controls.
    let stride=((corpus.len()-10)/8).max(1);
    let selected: Vec<_> = corpus.iter().enumerate().filter(|(i,_)| *i >= corpus.len()-10 || *i % stride == 0).collect();
    for profile in ["evaluation","benchmark-speed"] {
      for lane in ["copy","lz4"] {
        let wasm=fs::read(format!("{out}/{profile}-{lane}.wasm"))?;
        let artifact=hash(&wasm);
        let canister=pic.create_canister(); pic.add_cycles(canister,100_000_000_000_000);
        pic.install_canister(canister,wasm,vec![],None);
        for (_, (name,bytes)) in &selected {
          let compressed=compression_transport::compress(bytes);
          for operation in ["compress","decompress"] {
            let input=if operation == "compress" {bytes} else {&compressed};
            let expected=if lane=="copy" {input} else if operation=="compress" {&compressed} else {bytes};
            for rep in 0..9 {
              caller+=1;
              let result=pic.query_call(canister,Principal::from_slice(&caller.to_le_bytes()),operation,input.to_vec()).unwrap();
              assert_eq!(&result[8..],expected);
              let count=u64::from_le_bytes(result[..8].try_into().unwrap());
              writeln!(samples,"{}",json!({"profile":profile,"lane":lane,"case":name,"operation":operation,"repetition":rep,"warmup":rep==0,"caller":caller,"input_sha256":hash(input),"output_sha256":hash(expected),"body_instructions":count,"wasm_sha256":artifact}))?;
            }
          }
        }
      }
    }
    Ok(())
}

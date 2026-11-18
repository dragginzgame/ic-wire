// Current consumer measurement at opt-z and opt-3. Run from the repository root.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
const workload = process.argv[2] ?? 'wire-consumer';
assert(['wire-consumer', 'wire-float-consumer'].includes(workload), 'unknown workload');
assert(process.argv.length <= 3, 'expected at most one workload');
for (const name of Object.keys(process.env)) {
  if (['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'].includes(name) || name.startsWith('CARGO_PROFILE_') || /^CARGO_TARGET_.*_RUSTFLAGS$/.test(name)) {
    assert(!process.env[name], `unset ${name} for the fixed measurement`);
  }
}
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
function command(program, args, capture = false) {
  const result = spawnSync(program, args, { encoding:'utf8', stdio: capture ? ['ignore','pipe','inherit'] : ['ignore','inherit','inherit'] });
  if (result.error) throw result.error;
  assert.equal(result.status,0, `${program} ${args.join(' ')} failed`);
  return capture ? result.stdout.trim() : undefined;
}
const inputs = ['Cargo.toml','rust-toolchain.toml','qualification/Cargo.toml','qualification/Cargo.lock','qualification/ic-wire-consumer/Cargo.toml'];
function sources(directory) {
  for (const entry of readdirSync(directory,{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name))) {
    const path = join(directory,entry.name);
    if (entry.isDirectory()) sources(path);
    else inputs.push(path);
  }
}
for (const directory of ['crates/ic-wire/src','crates/ic-wire-derive/src','qualification/ic-wire-consumer/src']) sources(directory);
inputs.push('crates/ic-wire/Cargo.toml','crates/ic-wire-derive/Cargo.toml','qualification/ic-wire-consumer/measure-wasm.mjs');
const hashes = Object.fromEntries(inputs.sort().map(path => [path,sha(readFileSync(path))]));
mkdirSync('qualification/target',{recursive:true});
const output = mkdtempSync(resolve('qualification/target/wasm-measurement-'));
const report = { workload, compiler:command('rustc',['-Vv'],true), node:process.version,
  base_commit:command('git',['rev-parse','HEAD'],true), source_sha256:sha(JSON.stringify(hashes)),
  lock_sha256:hashes['qualification/Cargo.lock'], artifacts:[] };
writeFileSync(join(output,'inputs.json'),JSON.stringify(hashes,null,2)+'\n');
function sections(bytes) {
  let offset = 8, code = 0;
  function leb() {
    let value=0, shift=0, byte;
    do { byte=bytes[offset++]; value+=(byte&127)*2**shift; shift+=7; } while(byte&128);
    return value;
  }
  while(offset < bytes.length) {
    const id=bytes[offset++], size=leb();
    if(id===10) code+=size;
    offset+=size;
  }
  assert.equal(offset,bytes.length);
  return { total_bytes:bytes.length, code_payload_bytes:code, other_bytes:bytes.length-code };
}
for (const profile of ['wire-size','wire-speed']) {
  const target=join(output,profile);
  command('cargo',['build','--manifest-path','qualification/Cargo.toml','-p','ic-wire-consumer','--bin',workload,'--target','wasm32-unknown-unknown','--profile',profile,'--target-dir',target,'--locked']);
  const path=join(target,'wasm32-unknown-unknown',profile,`${workload}.wasm`);
  const bytes=readFileSync(path), module=new WebAssembly.Module(bytes);
  assert.deepEqual(WebAssembly.Module.imports(module),[]);
  assert.equal(new WebAssembly.Instance(module).exports.main(0,0),0);
  report.artifacts.push({profile,sha256:sha(bytes),...sections(bytes)});
}
writeFileSync(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({directory:output,...report},null,2));

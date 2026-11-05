// The matched runner supplies these modules in its isolated output directory.
#[rustfmt::skip]
mod before;
#[rustfmt::skip]
mod after;
use proc_macro2::TokenStream;
use std::{hint::black_box, time::Instant};
fn main() {
    println!("case,lane,sample,repetitions,host_ns_per_expansion");
    // Defaults are current behavior on both sides, not a one-sided research lane.
    for (case, query, args, item) in [
        ("unit-default", true, "contract = ECHO", "fn echo() {}"),
        (
            "scalar-default",
            true,
            "",
            "fn echo(value: u64) -> u64 { value }",
        ),
        (
            "scalar-explicit",
            true,
            "contract = ECHO, id = 7, request = REQUEST, response = RESPONSE",
            "fn echo(value: u64) -> u64 { value }",
        ),
        (
            "async-update",
            false,
            "contract = ECHO, id = 7, request = REQUEST, response = RESPONSE",
            "async fn echo(value: ApplicationRequest) -> Result<ApplicationResponse, ApplicationError> { handle(value).await }",
        ),
        (
            "borrowed-query",
            true,
            "borrow, contract = ECHO",
            "fn echo(value: &str) -> &str { value }",
        ),
        (
            "borrowed-update",
            false,
            "borrow, id = 7, request = REQUEST, response = RESPONSE",
            "fn echo(value: Request<'_>) -> ByteBuf { handle(value) }",
        ),
        (
            "renamed-composite",
            true,
            "composite, name = \"read-state\", crate = \"renamed::wire\"",
            "fn echo(value: u64) -> (u64, bool) { (value, true) }",
        ),
        (
            "local-collisions",
            true,
            "contract = ECHO",
            "#[cfg_attr(any(), inline)] fn echo(__wire_value: u64) -> u64 { let __WIRE_CONTRACT = __wire_value; __WIRE_CONTRACT }",
        ),
    ] {
        let item: TokenStream = item.parse().unwrap();
        let args: TokenStream = args.parse().unwrap();
        let before = before::expand(query, args.clone(), item.clone()).unwrap();
        let after = after::expand(query, args.clone(), item.clone()).unwrap();
        assert_eq!(
            before.to_string(),
            after.to_string(),
            "generated tokens changed: {case}"
        );
        for sample in 0..11 {
            let lanes = if sample % 2 == 0 {
                ["before", "after"]
            } else {
                ["after", "before"]
            };
            for lane in lanes {
                for _ in 0..8 {
                    let expanded = if lane == "before" {
                        before::expand(query, args.clone(), item.clone())
                    } else {
                        after::expand(query, args.clone(), item.clone())
                    };
                    black_box(expanded.unwrap());
                }
                let mut total = 0;
                let start = Instant::now();
                for _ in 0..1000 {
                    let result = if lane == "before" {
                        before::expand(query, args.clone(), item.clone())
                    } else {
                        after::expand(query, args.clone(), item.clone())
                    };
                    total += black_box(result.unwrap()).into_iter().count();
                }
                black_box(total);
                println!(
                    "{case},{lane},{sample},1000,{}",
                    start.elapsed().as_nanos() as f64 / 1000.0
                );
            }
        }
    }
}

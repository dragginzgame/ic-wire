mod before;
mod after;
use proc_macro2::TokenStream;
use std::{hint::black_box, time::Instant};
fn fresh_ident(source: &str, base: &str) -> syn::Ident {
    let mut name = base.to_owned();
    while source.contains(&name) { name.push('_'); }
    quote::format_ident!("{name}")
}
fn main() {
    println!("case,lane,sample,repetitions,host_ns_per_expansion");
    for (case, item) in [
        ("unit", "fn echo() {}"),
        ("scalar", "fn echo(value: u64) -> u64 { value }"),
        ("record", "async fn echo(value: ApplicationRequest) -> Result<ApplicationResponse, ApplicationError> { handle(value).await }"),
    ] {
        let item: TokenStream = item.parse().unwrap();
        let args: TokenStream = "contract = ECHO, id = 3346031142226386015u64, request = REQUEST, response = RESPONSE".parse().unwrap();
        let defaults: TokenStream = "contract = ECHO".parse().unwrap();
        for sample in 0..11 {
            // Alternate ordering to avoid consistently favouring one warmed lane.
            let lanes = if sample % 2 == 0 { ["before", "after", "defaults"] } else { ["defaults", "after", "before"] };
            for lane in lanes {
                let mut total = 0;
                let start = Instant::now();
                for _ in 0..1000 {
                    let selected = if lane == "defaults" { defaults.clone() } else { args.clone() };
                    let result = if lane == "before" { before::expand(true, selected, item.clone()) }
                        else { after::expand(true, selected, item.clone()) };
                    total += black_box(result.unwrap()).into_iter().count();
                }
                black_box(total);
                println!("{case},{lane},{sample},1000,{}", start.elapsed().as_nanos() as f64 / 1000.0);
            }
        }
    }
}

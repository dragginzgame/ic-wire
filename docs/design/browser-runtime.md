# Candidate TypeScript runtime

Destination: one interpreter for the schema subset admitted by ValidatedRegistry,
used by generated payload and request/response modules. No SDK, endpoint,
authentication, retries, deployment or generated per-record byte writers belong
here. Rust continues to use one published Serde/Postcard codec.

The admitted runtime kinds are unit, bool, u/i (8/16/32/64/128), bytes, text,
packed-i16, list, option, newtype, record, enum and named ref. Metadata and limits
are generated trusted input; this is not an untrusted-schema service. The format
marker is `ic-wire-json/1`. Custom validators and other reference kinds remain
outside the product contract until host admission and runtime qualification exist.

Reviewed reference: Core 0.22.4 at
`2dc434b10bec2fca92345d940dbfedc05993e9d5`,
`crates/ic-wire/src/typescript/runtime.ts`, under the retained repository license.
The pinned source SHA-256 is
`9c01a1b4f2835c67923c817c810fa061b89227e2120f829b3899682a45eeeb93`.
Selected reuse comprises the error taxonomy, primitive varint/ZigZag rules,
strict object checks, traversal accounting, bounded buffer growth, UTF-8 checks,
packed-i16 custody and frame admission. Unsupported schema branches, custom checks,
float/generic-packed specialization and producer release machinery are excluded.
The existing independent literal and refusal fixtures are the initial contract
oracles, not evidence that untested parts of the source are qualified.

Preserve emitted bytes, primitive acceptance, typed failures, budget ordering,
selected Uint8Array spans, null-prototype records and owned bulk results. Alias
resolution spends no value node; each concrete value does. JS byte/node/depth
allowances are not Rust allocation accounting or a process heap cap. Encoding
grows bounded private output, as the reference does; it does not promise zero
output allocation on invalid input. Trusted getters/proxies are not sandboxed.

Qualification must execute the same generated clients against candidate and
published-reference sources, compare outputs and literal Rust decode results,
and exercise malformed inputs and exact budgets. Browser-engine execution,
performance, installed distribution and consumer
adoption remain separate gates. No performance improvement or consumer cutover
is authorized by implementing this runtime.

The candidate uses one BigInt varint path instead of retaining separate small-number
fast paths. This is an implementation simplification, not a throughput claim.
Node qualification runs generated payload/frame clients against both sources and
compares all one- and two-byte u16/i16 inputs, all single-byte bool/u8/i8 inputs,
selected wider overflow/truncation groups and unpaired UTF-16 surrogates. Existing
literal, custody and exact-budget fixtures execute against both runtimes.
The alternate native-endian branch remains unexecuted on this little-endian host.

`make check-typescript` adds strict TypeScript 5.9.3 checking of the runtime and
generated payload/frame modules as ES modules, with unchecked index access and
exact optional property checks. Compile-only consumer fixtures assert the typed
encode/decode contract and expected errors for invalid input shapes. This is
static evidence for those generated clients, not browser execution proof.

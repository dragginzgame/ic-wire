# IC Wire New

Read TOKO-TOOLING.md, README.md and docs/DESIGN.md before implementation.
The installed Toko Tooling baseline is adopted; this overlay retains Wire's
product scope and stricter safety constraints.

- This is an authorized local replacement evaluation, not a deployed replacement.
  The sibling `../ic-wire` is read-only reference and remains the production owner.
- Reduce independent responsibilities and execution paths. Do not transplant the
  old implementation or its release system merely to obtain a working workspace.
  Reuse requires a destination contract, provenance and focused qualification.
- Preserve one production Serde/Postcard codec unless a separately justified
  design decision changes it. No private Postcard fork, parallel codec lanes or
  automatically rewritten consumer Serde representations.
- Keep codec code Wasm-safe and independent of SDKs, native tools and Candid.
  Endpoint execution, transport, authentication, retries and deployment belong
  downstream. Do not restore Wire Endpoint or an executor.
- Platform Types owns Principal/Account identities and Serde implementations.
  Wire owns optional host schema integration; keep codec-only graphs small.
- Preserve pre-allocation admission and typed malformed-input errors. Account for
  trusted custom Serde work and consumer-owned collection normalization explicitly.
- The user authorized a hard cut: legacy API, wire, schema, acceptance and budget
  compatibility are not requirements. Keep one current implementation without
  versioned compatibility lanes. Record intentional contract changes and qualify
  safety and cross-language behavior. Actual deployment and persisted-state
  operations remain separately authorized actions.
- Avoid new abstraction layers and per-type codec specialization without measured
  justification. Prioritize total uncompressed Wasm at opt-z; report opt-3 losses,
  IC instructions, allocation and payload bytes separately. Do not optimize gzip.
- Keep product packages in a virtual root under crates/ and real qualification
  packages in a separate qualification/ workspace. Add no placeholder packages.
  Candid is an optional research comparator in private dev-only qualification.
  Use published Host, Build and Testkit for their owned native responsibilities.
- Tooling is installed through the authenticated Toko Tooling 0.44.3 exporter;
  `.toko-tooling.snapshot` records its source and owned bytes. Detailed reference
  policy remains pinned at revision 626da7cd9626cc2e152594862a2d82ea82ef51da;
  do not follow a moving sibling baseline. The compiler is Rust 1.100.0.
  `make toko-tooling-check` admits the member declaration, Cargo.lock and snapshot;
  `make snapshot-check` verifies just the installed bytes.
- Keep lasting design decisions small and current; do not copy evidence trees,
  historical journals or broad orchestration before there is a real caller.
- This initialization authorizes no commit, remote creation, publication,
  deployment, consumer migration or deletion of the original repository.

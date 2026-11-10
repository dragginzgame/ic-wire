# IC Wire documentation

## Product guides

- [Runtime contract](guides/runtime.md): codec bytes, bounds and acceptance.
- [Architecture and ownership](guides/design.md).
- [Field selection](guides/packed-fields.md): bytes, Packed and explicit adapters.
- [Typed endpoints](guides/endpoints.md) and [SDK integration](guides/sdk-integration.md).
- [Unsigned magnitudes](guides/unsigned-magnitude.md).
- [Development commands](guides/development.md).
- [Release identities](guides/stable-identities.md) and [group validation](guides/group-validation.md).
- [Consumer qualification](guides/consumer-fixtures.md).

## Measurements and research

- [Latest Candid comparison](benchmarks/candid-comparison.md).
- [Evaluation scope](benchmarks/evaluation.md) and [maintained probes](benchmarks/performance.md).
- [Topic findings](evidence/README.md): measured changes, rejected experiments and limitations.
- [Historical reports](history/README.md): earlier releases and retired implementations.

Each result belongs to its recorded source and inputs. A historical report is
not a supported API or a current performance guarantee. Raw runs belong in
`target/evidence/`, not this documentation tree.

## Current work and delivery

- [Current handoff](status/current.md).
- [Release ledgers](../CHANGELOG.md) and [delivery receipts](../releases/receipts/README.md).

## Toko Tooling references

Shared reference documentation stays upstream at reviewed, commit-pinned URLs:

- [Adoption](https://github.com/dragginzgame/toko-tooling/blob/12459b6029115b830b8477879d3bea2f030b6e63/docs/adoption/consuming-snapshots.md) and [local tool setup](https://github.com/dragginzgame/toko-tooling/blob/12459b6029115b830b8477879d3bea2f030b6e63/docs/adoption/local-setup.md).
- [Release tooling](https://github.com/dragginzgame/toko-tooling/blob/12459b6029115b830b8477879d3bea2f030b6e63/docs/releases/releases.md).
- [Tool inventory](https://github.com/dragginzgame/toko-tooling/blob/12459b6029115b830b8477879d3bea2f030b6e63/docs/tools/tool-inventory.md).
- [Validation helpers](https://github.com/dragginzgame/toko-tooling/blob/12459b6029115b830b8477879d3bea2f030b6e63/docs/validation/verification-helpers.md).
- [Engineering principles](https://github.com/dragginzgame/toko-tooling/blob/12459b6029115b830b8477879d3bea2f030b6e63/docs/principles/README.md) and [root baseline](../TOKO-TOOLING.md).

Only the root baseline and required executable files are installed locally.
Product guides, benchmarks, findings, history and status remain consumer-owned.

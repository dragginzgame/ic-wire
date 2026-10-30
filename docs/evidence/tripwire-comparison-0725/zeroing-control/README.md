# Initial zero-filled ingress control

The first standalone fixture used `vec![0; length]` followed by the IC argument
copy. Current Platform's production `cdk::api::msg_arg_data()` initializes the
final buffer directly. The [headline study](../README.md) was therefore rerun
with that helper for both Wire and Candid, using the same logical inputs,
compiler, profiles and instrumented server. No production optimization was made.

These 1,280 checked physical samples remain evidence for the initial fixture,
not the current production-helper result. [restore-fixture.patch](restore-fixture.patch)
transforms the current fixture back into this control; its frozen Cargo.lock,
source receipt, per-profile summaries and raw samples are retained here.
The correction also enables Platform's CDK feature and uses a new source path;
do not attribute every artifact-byte change solely to the removed zeroing pass.

At opt-3, borrowed Wire's one MiB prefix read changes from 5,143,153.5 to
4,094,977.25 instructions. Tripwire selective remains 250,590. The remaining
16.34× gap comes from different input-access work, not a forgotten zeroing
optimization in Wire's production Platform adapter.

Original raw builds and logs: `/tmp/wire-tripwire-0725/`.

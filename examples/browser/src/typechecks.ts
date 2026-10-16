import * as apply from '../generated/apply.js';
import * as inspect from '../generated/inspect.js';
// Compiled but never bundled; unused expect-error directives fail checking.
if (false) {
  const request: apply.Request = { subject: new Uint8Array(), audience: new Uint8Array(), expiry: 1n, id: 1n, amount: 1n };
  // @ts-expect-error u64 inputs require bigint even for small values.
  apply.encodeRequest({ ...request, amount: 7 });
  // @ts-expect-error principal bytes require Uint8Array.
  apply.encodeRequest({ ...request, subject: [0,1] });
  // @ts-expect-error Option uses an explicit Some wrapper.
  inspect.encodeRequest({ labels: [], correlation: 7n });
  // @ts-expect-error enum tags are restricted to the declared alternatives.
  apply.encodeResponse({ err: { tag: 'Unknown', value: { id: 1n, balance: 0n } } });
}

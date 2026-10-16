import { encodeRequest } from './client.js';
// Compiled but never imported by the browser bundle. Unused @ts-expect-error
// directives fail TypeScript checking if the public wrapper loses these bounds.
if (false) {
  // @ts-expect-error u64 input must be bigint, including small integers.
  encodeRequest({ account: 7, revision: 42n, payload: new Uint8Array() });
  // @ts-expect-error byte payloads use Uint8Array rather than unchecked arrays.
  encodeRequest({ account: 7n, revision: 42n, payload: [0, 1] });
}

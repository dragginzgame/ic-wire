import { count } from './payloads.js';
export class Counter {
  constructor(readonly value: bigint) {
    if (value < 0n) throw new RangeError('negative counter');
  }
}
export const counter = {
  encode: (value: Counter): Uint8Array => count.encode(value.value),
  decode: (bytes: Uint8Array): Counter => new Counter(count.decode(bytes))
};

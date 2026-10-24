import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

// Simulate numeric typed-array storage, not a big-endian JavaScript engine.
// Use DataView as the independent endian oracle; do not reuse runtime swapWords.
function bigEndianArray(Native, getter, setter) {
  return class BigEndianArray extends Native {
    constructor(input, ...args) {
      if (input instanceof ArrayBuffer) {
        const view = new DataView(input);
        const values = Array.from({length: input.byteLength / Native.BYTES_PER_ELEMENT},
          (_, i) => view[getter](i * Native.BYTES_PER_ELEMENT, false));
        super(new Native(values).buffer, ...args);
      } else super(input, ...args);
    }
    get buffer() {
      const values = new Native(super.buffer);
      const buffer = new ArrayBuffer(super.buffer.byteLength);
      const view = new DataView(buffer);
      values.forEach((value, i) => view[setter](i * Native.BYTES_PER_ELEMENT, value, false));
      return buffer;
    }
  };
}
const specs = [
  ['i16', 'Int16Array', 'getInt16', 'setInt16', [-32768, 1, 32767]],
  ['u16', 'Uint16Array', 'getUint16', 'setUint16', [0, 1, 65535]],
  ['i32', 'Int32Array', 'getInt32', 'setInt32', [-2147483648, 1, 2147483647]],
  ['u32', 'Uint32Array', 'getUint32', 'setUint32', [0, 1, 4294967295]],
  ['i64', 'BigInt64Array', 'getBigInt64', 'setBigInt64', [-(1n<<63n), 1n, (1n<<63n)-1n]],
  ['u64', 'BigUint64Array', 'getBigUint64', 'setBigUint64', [0n, 1n, (1n<<64n)-1n]],
];
const source = await readFile(process.argv[2], 'utf8');
const prelude = `${bigEndianArray.toString()}\n` + specs.map(([, name, get, set]) =>
  `const ${name} = bigEndianArray(globalThis.${name}, '${get}', '${set}');`).join('\n');
const { createCodec, WireError, arrays } = await import('data:text/javascript;base64,' +
  Buffer.from(prelude + '\n' + source + '\nexport const arrays = {' + specs.map(x=>x[1]).join(',') + '};').toString('base64'));
assert.deepEqual(Array.from(new Uint8Array(new arrays.Uint16Array([0x1234]).buffer)), [0x12, 0x34]);
const header = [73,87,1,0,0,0,0,0,0,0,0];
let cases = 0;
for (const [word, name, , setter, values] of [...specs, ['hash32', 'Uint8Array', '', '', Array.from({length:32}, (_,i)=>i)]]) {
  const Ctor = arrays[name] ?? Uint8Array;
  for (const count of [0, 1, 3]) {
    const selected = word === 'hash32' ? Array.from({length: count * 32}, (_,i)=>i % 32) : values.slice(0,count);
    const value = new Ctor(selected);
    const expected = new Uint8Array(value.byteLength);
    if (word === 'hash32') expected.set(selected);
    else selected.forEach((v,i) => new DataView(expected.buffer)[setter](i * Ctor.BYTES_PER_ELEMENT, v, true));
    for (const schema of [{k:'packed',word}, ...(word === 'i16' ? [{k:'packed-i16',max:3}] : [])]) {
      const codec = createCodec({id:'1',request:schema,response:schema,requestBytes:256,responseBytes:256,requestNodes:16,responseNodes:16,requestDepth:8,responseDepth:8});
      for (const direction of ['Request','Response']) {
        const frame = Uint8Array.from([...header, expected.length, ...expected]);
        frame[10] = direction === 'Request' ? 0 : 1;
        assert.deepEqual(codec['encode'+direction](value), frame);
        assert.deepEqual(Array.from(value), selected); // encoding must not mutate input
        const padded = new Uint8Array(frame.length+3); padded.set(frame,1);
        const decoded = codec['decode'+direction](padded.subarray(1,-2));
        padded.fill(0);
        assert.deepEqual(Array.from(decoded), selected);
        const malformed = Uint8Array.from([...header,1,0]); malformed[10] = frame[10];
        assert.throws(()=>codec['decode'+direction](malformed), WireError);
        cases++;
      }
    }
  }
}
console.log(`Simulated big-endian packed codec: ${cases} cases passed (not a native big-endian engine).`);

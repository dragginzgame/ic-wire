// The framed byte oracle is independent of the Rust encoder under measurement.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const template = readFileSync(new URL('guest.rs.in', import.meta.url), 'utf8');
const integers = ['u8', 'u16', 'u32', 'u64'];
const method = i => `scale_${i}`;
const fieldId = (i, shape) => shape === 'layouts' ? 1 : i + 1;

export function guest(count, shape, callerScratch, metered, explicitSparse) {
  const source = template.replaceAll('#[wire(crate = "wire")]', explicitSparse ? '#[wire(crate = "wire", sparse)]' : '#[wire(crate = "wire")]')
    .replace('/* COUNTER */', metered ? 'raw::msg_reply_data_append(&raw::instruction_counter().to_le_bytes());' : '')
    .replace(' /* SCRATCH */', callerScratch ? ', &mut [0; 256]' : '');
  return source + Array.from({ length: count }, (_, i) => {
    const layout = shape === 'layouts' ? i : i % 16;
    const types = Array.from({ length: 5 }, (_, digit) => integers[(layout >> (2 * digit)) & 3]);
    return `endpoint!(m${i}, "${method(i)}", ${fieldId(i, shape)}, ${types.join(', ')});\n`;
  }).join('');
}

function header(name, direction) {
  let hash = 0xcbf29ce484222325n;
  for (const byte of Buffer.from('ic-wire:method:' + name)) {
    hash = BigInt.asUintN(64, (hash ^ BigInt(byte)) * 0x100000001b3n);
  }
  const frame = Buffer.from([73, 87, 0, 0, 0, 0, 0, 0, 0, 0, direction]);
  frame.writeBigUInt64LE(hash, 2);
  return [...frame];
}

export function cases(i, shape) {
  const name = method(i);
  const id = fieldId(i, shape);
  const tag = id < 128 ? [id] : [(id & 127) | 128, id >> 7];
  const known = [...tag, 1, 17];
  const unknown = [255, 15, 1, 9]; // id 2047, above all generated IDs
  return [
    ['absent', [], 0], ['present', known, 17], ['unknown', unknown, 0],
    ['both', [...known, ...unknown], 17], ['duplicate', [...known, ...known], null],
    ['descending', [...unknown, ...known], null], ['truncated', [...tag, 5, 1], null],
  ].map(([kind, ext, value]) => ({
    name, kind, request: [...header(name, 0), 42, 5, 6, 7, ext.length, ...ext],
    reply: value === null ? null : [...header(name, 1), value, 5, 6, 7, tag.length + 2, ...tag, 1, 42],
  }));
}

export function verify(bytes, count, shape) {
  let memory, input, reply, replied;
  const module = new WebAssembly.Module(bytes);
  const imports = {
    msg_arg_data_size: () => input.length,
    msg_arg_data_copy(dst, offset, len) { new Uint8Array(memory.buffer, dst, len).set(input.slice(offset, offset + len)); },
    msg_reply_data_append(ptr, len) { reply.push(...new Uint8Array(memory.buffer, ptr, len)); },
    msg_reply() { replied++; },
    trap() { throw new Error('IC trap'); },
  };
  const names = WebAssembly.Module.exports(module).map(item => item.name).filter(n => n.startsWith('canister_')).sort();
  assert.deepEqual(names, Array.from({ length: count }, (_, i) => 'canister_query ' + method(i)).sort());
  for (let i = 0; i < count; i++) {
    const { exports } = new WebAssembly.Instance(module, { ic0: imports });
    memory = exports.memory;
    for (const test of cases(i, shape)) {
      input = test.request;
      reply = [];
      replied = 0;
      const call = () => exports['canister_query ' + test.name]();
      if (test.reply === null) assert.throws(call, /IC trap/);
      else { call(); assert.deepEqual(reply, test.reply); assert.equal(replied, 1); }
    }
  }
}

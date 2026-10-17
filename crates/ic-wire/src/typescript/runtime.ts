// Canonical Postcard inside the IC Wire envelope. Metadata and scalar validators
// are trusted consumer declarations. Traversal limits do not bound JS engine heap.
export class WireError extends Error {
  constructor(readonly code: 'limit' | 'malformed' | 'identity' | 'value') {
    super(`wire ${code}`);
  }
}
type Field = { name: string; value: Schema };
export type Schema = ( { k: 'unit' | 'bool' }
  | { k: 'bytes'; min?: number; max?: number }
  | { k: 'text'; max?: number }
  | { k: 'u' | 'i' | 'float'; bits: number; min?: number; max?: number }
  | { k: 'list'; max?: number; item: Schema }
  | { k: 'option'; item: Schema }
  | { k: 'newtype'; item: Schema }
  | { k: 'array'; max: number; item: Schema }
  | { k: 'tuple'; items: readonly Schema[] }
  | { k: 'result'; ok: Schema; err: Schema }
  | { k: 'record'; fields: readonly Field[] }
  | { k: 'enum'; variants: readonly (Field & { id: number })[] }
  | { k: 'ref'; name: string }) & { check?: string };
type Method = { id: string; request: Schema; response: Schema; requestBytes: number; responseBytes: number; requestNodes: number; responseNodes: number; requestDepth: number; responseDepth: number };
type Types = Readonly<Record<string, Schema>>;
type Checks = Readonly<Record<string, (value: unknown) => void>>;
const fail = (code: WireError['code']): never => { throw new WireError(code); };
const utf8 = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
const utf16 = new TextEncoder();
function object(value: unknown, keys: readonly string[]): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return fail('value');
  const own = Reflect.ownKeys(value);
  if (own.length !== keys.length || keys.some(k => !Object.hasOwn(value, k))) return fail('value');
  return value as Record<string, unknown>;
}
class Budget {
  constructor(public remaining: number, readonly depth: number) {}
  visit(depth: number) { if (--this.remaining < 0 || depth > this.depth) fail('limit'); }
  length(length: number) { if (length > this.remaining) fail('limit'); }
}
// Resolve aliases without spending a value node, but refuse missing or cyclic
// aliases. Recursive values must pass through a concrete node and its budget.
function resolve(schema: Schema, types: Types): Schema {
  let remaining = Object.keys(types).length;
  while (schema.k === 'ref') {
    if (remaining-- === 0 || !Object.hasOwn(types, schema.name)) return fail('value');
    schema = types[schema.name];
  }
  return schema;
}
function validate(schema: Schema, value: unknown, checks: Checks): void {
  if (schema.check !== undefined) {
    if (!Object.hasOwn(checks, schema.check)) fail('value');
    checks[schema.check](value);
  }
}
class Writer {
  readonly bytes: Uint8Array;
  offset = 11;
  constructor(size: number, id: bigint, direction: number) {
    this.bytes = new Uint8Array(size);
    this.bytes.set([73, 87]);
    new DataView(this.bytes.buffer).setBigUint64(2, id, true);
    this.bytes[10] = direction;
  }
  byte(value: number) { if (this.offset === this.bytes.length) fail('limit'); this.bytes[this.offset++] = value; }
  varint(value: bigint) {
    do { const byte = Number(value & 127n); value >>= 7n; this.byte(value ? byte | 128 : byte); } while (value);
  }
  raw(bytes: Uint8Array) {
    if (bytes.length > this.bytes.length - this.offset) fail('limit');
    this.bytes.set(bytes, this.offset); this.offset += bytes.length;
  }
}
class Reader {
  offset = 11;
  constructor(readonly bytes: Uint8Array) {}
  byte(): number { if (this.offset === this.bytes.length) return fail('malformed'); return this.bytes[this.offset++]; }
  varint(bits: number): bigint {
    let value = 0n;
    for (let i = 0; i < Math.ceil(bits / 7); i++) {
      const byte = this.byte(); value |= BigInt(byte & 127) << BigInt(i * 7);
      if (!(byte & 128)) {
        if (value >= (1n << BigInt(bits)) || (i > 0 && byte === 0)) return fail('malformed');
        return value;
      }
    }
    return fail('malformed');
  }
  length(max: number): number { const n = this.varint(64); if (n > BigInt(max)) return fail('limit'); return Number(n); }
  raw(length: number): Uint8Array {
    if (length > this.bytes.length - this.offset) return fail('malformed');
    const value = this.bytes.subarray(this.offset, this.offset + length); this.offset += length; return value;
  }
}
function write(s: Schema, value: unknown, out: Writer, budget: Budget, depth: number, types: Types, checks: Checks): void {
  s = resolve(s, types);
  validate(s, value, checks);
  budget.visit(depth);
  const child = (schema: Schema, item: unknown) => write(schema, item, out, budget, depth + 1, types, checks);
  switch (s.k) {
    case 'unit': if (value !== null) fail('value'); return;
    case 'bool': if (typeof value !== 'boolean') fail('value'); out.byte(value ? 1 : 0); return;
    case 'u': case 'i': {
      if (s.bits <= 32 ? typeof value !== 'number' || !Number.isSafeInteger(value) : typeof value !== 'bigint') fail('value');
      const n = BigInt(value as number | bigint), bits = BigInt(s.bits), signed = s.k === 'i';
      const min = signed ? -(1n << (bits - 1n)) : 0n, max = (1n << (bits - (signed ? 1n : 0n))) - 1n;
      if (n < min || n > max) fail('limit');
      if (s.min !== undefined && n < BigInt(s.min) || s.max !== undefined && n > BigInt(s.max)) fail('value');
      if (s.bits === 8) { out.byte(Number(n & 255n)); return; }
      out.varint(signed ? (n << 1n) ^ (n >> (bits - 1n)) : n); return;
    }
    case 'float': {
      if (typeof value !== 'number' || !Number.isFinite(value) || (s.bits === 32 && !Object.is(Math.fround(value), value))) fail('value');
      const bytes = new Uint8Array(s.bits / 8), view = new DataView(bytes.buffer);
      if (s.bits === 32) view.setFloat32(0, value as number, true); else view.setFloat64(0, value as number, true);
      out.raw(bytes); return;
    }
    case 'bytes': {
      if (!(value instanceof Uint8Array)) fail('value');
      const bytes = value as Uint8Array;
      if (s.min !== undefined && bytes.length < s.min) fail('value');
      if (s.max !== undefined && bytes.length > s.max) fail('limit');
      out.varint(BigInt(bytes.length)); out.raw(bytes); return;
    }
    case 'text': {
      if (typeof value !== 'string') fail('value');
      const text = value as string, remaining = Math.min(s.max ?? out.bytes.length, out.bytes.length - out.offset);
      if (text.length > remaining) fail('limit');
      const buffer = new Uint8Array(remaining), result = utf16.encodeInto(text, buffer);
      if (result.read !== text.length) fail('limit');
      const bytes = buffer.subarray(0, result.written);
      if (utf8.decode(bytes) !== text) fail('value');
      out.varint(BigInt(bytes.length)); out.raw(bytes); return;
    }
    case 'newtype': child(s.item, value); return;
    case 'list': case 'array': case 'tuple': {
      if (!Array.isArray(value)) fail('value');
      const values = value as unknown[];
      if (s.k === 'array' && values.length !== s.max || s.k === 'list' && values.length > (s.max ?? budget.remaining)) fail('limit');
      if (s.k === 'tuple' && values.length !== s.items.length) fail('value');
      budget.length(values.length);
      if (s.k === 'list') out.varint(BigInt(values.length));
      for (let i = 0; i < values.length; i++) {
        if (!Object.hasOwn(values, i)) fail('value');
        child(s.k === 'tuple' ? s.items[i] : s.item, values[i]);
      }
      return;
    }
    case 'option': {
      if (value === null) { out.byte(0); return; }
      const record = object(value, ['some']); out.byte(1); child(s.item, record.some); return;
    }
    case 'result': {
      const key = value && typeof value === 'object' && Object.hasOwn(value, 'ok') ? 'ok' : 'err';
      const record = object(value, [key]); out.byte(key === 'ok' ? 0 : 1); child(s[key], record[key]); return;
    }
    case 'record': {
      const record = object(value, s.fields.map(f => f.name));
      for (const field of s.fields) child(field.value, record[field.name]); return;
    }
    case 'enum': {
      const record = object(value, ['tag', 'value']);
      const variant = s.variants.find(v => v.name === record.tag); if (!variant) return fail('value');
      out.varint(BigInt(variant.id)); child(variant.value, record.value); return;
    }
  }
}
function read(s: Schema, input: Reader, budget: Budget, depth: number, types: Types, checks: Checks): unknown {
  s = resolve(s, types);
  const value = readValue(s, input, budget, depth, types, checks);
  validate(s, value, checks);
  return value;
}
function readValue(s: Schema, input: Reader, budget: Budget, depth: number, types: Types, checks: Checks): unknown {
  budget.visit(depth);
  const child = (schema: Schema) => read(schema, input, budget, depth + 1, types, checks);
  switch (s.k) {
    case 'unit': return null;
    case 'bool': { const n = input.byte(); if (n > 1) return fail('malformed'); return n === 1; }
    case 'u': case 'i': {
      let n = s.bits === 8 ? BigInt(input.byte()) : input.varint(s.bits);
      if (s.k === 'i') n = s.bits === 8 ? (n >= 128n ? n - 256n : n) : (n >> 1n) ^ -(n & 1n);
      if (s.min !== undefined && n < BigInt(s.min) || s.max !== undefined && n > BigInt(s.max)) fail('malformed');
      return s.bits <= 32 ? Number(n) : n;
    }
    case 'float': {
      const bytes = input.raw(s.bits / 8), view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
      const value = s.bits === 32 ? view.getFloat32(0, true) : view.getFloat64(0, true);
      if (!Number.isFinite(value)) return fail('malformed'); return value;
    }
    case 'bytes': {
      const length = input.length(s.max ?? input.bytes.length - input.offset);
      if (s.min !== undefined && length < s.min) fail('malformed');
      return input.raw(length).slice();
    }
    case 'text': {
      const text = utf8.decode(input.raw(input.length(Math.min(s.max ?? input.bytes.length, input.bytes.length - input.offset))));
      return text;
    }
    case 'newtype': return child(s.item);
    case 'list': case 'array': case 'tuple': {
      const length = s.k === 'list' ? input.length(Math.min(s.max ?? budget.remaining, budget.remaining)) : s.k === 'array' ? s.max : s.items.length;
      budget.length(length);
      const values: unknown[] = [];
      for (let i = 0; i < length; i++) values.push(child(s.k === 'tuple' ? s.items[i] : s.item));
      return values;
    }
    case 'option': {
      const tag = input.byte(); if (tag === 0) return null; if (tag !== 1) return fail('malformed');
      return { some: child(s.item) };
    }
    case 'result': {
      const tag = input.byte(); if (tag > 1) return fail('malformed');
      return tag === 0 ? { ok: child(s.ok) } : { err: child(s.err) };
    }
    case 'record': {
      const value = Object.create(null) as Record<string, unknown>;
      for (const field of s.fields) value[field.name] = child(field.value);
      return value;
    }
    case 'enum': {
      const id = input.varint(32), variant = s.variants.find(v => BigInt(v.id) === id);
      if (!variant) return fail('malformed');
      return { tag: variant.name, value: child(variant.value) };
    }
  }
}
// Accept only generated trusted metadata. No SDK/auth/transport policy lives here.
export function createCodec(metadata: unknown, registry: unknown = {}, checks: Checks = {}) {
  const meta = metadata as Method, types = registry as Types, id = BigInt(meta.id);
  if (id < 0n || id >= (1n << 64n)) fail('value');
  for (const value of [meta.requestBytes, meta.responseBytes, meta.requestNodes, meta.responseNodes, meta.requestDepth, meta.responseDepth]) {
    if (!Number.isSafeInteger(value) || value < 1) fail('value');
  }
  if (meta.requestBytes < 11 || meta.responseBytes < 11) fail('value');
  function encode(value: unknown, direction: 0 | 1): Uint8Array {
    const out = new Writer(direction ? meta.responseBytes : meta.requestBytes, id, direction);
    write(direction ? meta.response : meta.request, value, out, new Budget(direction ? meta.responseNodes : meta.requestNodes, direction ? meta.responseDepth : meta.requestDepth), 1, types, checks);
    return out.bytes.slice(0, out.offset);
  }
  function decode(bytes: Uint8Array, direction: 0 | 1): unknown {
    if (!(bytes instanceof Uint8Array)) return fail('value');
    if (bytes.length > (direction ? meta.responseBytes : meta.requestBytes)) return fail('limit');
    if (bytes.length < 11) return fail('malformed');
    if (bytes[0] !== 73 || bytes[1] !== 87 || bytes[10] !== direction || new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getBigUint64(2, true) !== id) return fail('identity');
    try {
      const input = new Reader(bytes), value = read(direction ? meta.response : meta.request, input, new Budget(direction ? meta.responseNodes : meta.requestNodes, direction ? meta.responseDepth : meta.requestDepth), 1, types, checks);
      if (input.offset !== bytes.length) return fail('malformed');
      return value;
    } catch (error) { if (error instanceof WireError) throw error; return fail('malformed'); }
  }
  return { encodeRequest: (value: unknown) => encode(value, 0), decodeRequest: (bytes: Uint8Array) => decode(bytes, 0), encodeResponse: (value: unknown) => encode(value, 1), decodeResponse: (bytes: Uint8Array) => decode(bytes, 1) };
}

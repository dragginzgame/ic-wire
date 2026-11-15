// Candidate interpreter for the admitted host schema subset. See
// docs/design/browser-runtime.md for the pinned reference and reuse contract.
// Generated metadata is trusted; allowances do not bound the JS engine heap.
export const schemaFormat = 'ic-wire-json/1';
export class WireError extends Error {
  constructor(readonly code: 'limit' | 'malformed' | 'identity' | 'value') { super(`wire ${code}`); }
}
type Field = { name: string; value: Schema };
export type Schema = { k: 'unit' | 'bool' }
  | { k: 'u' | 'i'; bits: number }
  | { k: 'bytes'; min?: number; max?: number }
  | { k: 'text'; max?: number }
  | { k: 'packed-i16'; max: number }
  | { k: 'list'; item: Schema; max?: number }
  | { k: 'option' | 'newtype'; item: Schema }
  | { k: 'record'; fields: readonly Field[] }
  | { k: 'enum'; variants: readonly (Field & { id: number })[] }
  | { k: 'ref'; name: string };
type Types = Readonly<Record<string, Schema>>;
type Payload = { schema: Schema; bytes: number; nodes: number; depth: number };
type Method = { id: string; request: Schema; response: Schema; requestBytes: number; responseBytes: number; requestNodes: number; responseNodes: number; requestDepth: number; responseDepth: number };
const fail = (code: WireError['code']): never => { throw new WireError(code); };
const utf8 = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
const utf16 = new TextEncoder();
const littleEndian = new Uint8Array(new Uint16Array([1]).buffer)[0] === 1;
function swap16(bytes: Uint8Array): void {
  for (let i = 0; i < bytes.length; i += 2) { const low = bytes[i]!; bytes[i] = bytes[i + 1]!; bytes[i + 1] = low; }
}
function object(value: unknown, keys: readonly (string | Field)[]): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return fail('value');
  if (Reflect.ownKeys(value).length !== keys.length || keys.some(key => !Object.hasOwn(value, typeof key === 'string' ? key : key.name))) return fail('value');
  return value as Record<string, unknown>;
}
class Budget {
  constructor(public remaining: number, readonly depth: number) {}
  visit(depth: number) { if (--this.remaining < 0 || depth > this.depth) fail('limit'); }
  length(length: number) { if (length > this.remaining) fail('limit'); }
}
function resolve(schema: Schema, types: Types): Schema {
  // Reference checkpoint algorithm: finite alias walk, no per-value registry copy.
  let checkpoint: string | undefined, distance = 0, span = 1;
  while (schema.k === 'ref') {
    if (schema.name === checkpoint || !Object.hasOwn(types, schema.name)) return fail('value');
    if (distance++ === span) { checkpoint = schema.name; span *= 2; distance = 1; }
    const next = types[schema.name];
    if (next === undefined) return fail('value');
    schema = next;
  }
  return schema;
}
class Writer {
  bytes: Uint8Array;
  constructor(readonly limit: number, public offset = 0) { this.bytes = new Uint8Array(Math.min(limit, 64)); }
  private grow(required: number) {
    if (required > this.limit) fail('limit');
    const next = new Uint8Array(Math.min(this.limit, Math.max(required, this.bytes.length * 2)));
    next.set(this.bytes); this.bytes = next;
  }
  header(id: bigint, direction: number) {
    this.bytes.set([73, 87]); new DataView(this.bytes.buffer).setBigUint64(2, id, true); this.bytes[10] = direction;
  }
  byte(value: number) { if (this.offset === this.bytes.length) this.grow(this.offset + 1); this.bytes[this.offset++] = value; }
  varint(value: bigint) {
    do { const byte = Number(value & 127n); value >>= 7n; this.byte(value ? byte | 128 : byte); } while (value);
  }
  raw(bytes: Uint8Array) {
    if (bytes.length > this.bytes.length - this.offset) this.grow(this.offset + bytes.length);
    this.bytes.set(bytes, this.offset); this.offset += bytes.length;
  }
  finish(): Uint8Array { return this.offset === this.bytes.length ? this.bytes : this.bytes.slice(0, this.offset); }
}
class Reader {
  readonly view: DataView;
  constructor(readonly bytes: Uint8Array, public offset = 0) { this.view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength); }
  byte(): number { const value = this.bytes[this.offset++]; if (value === undefined) return fail('malformed'); return value; }
  varint(bits: number): bigint {
    let value = 0n;
    for (let i = 0; i < Math.ceil(bits / 7); i++) {
      const byte = this.byte(); value |= BigInt(byte & 127) << BigInt(i * 7);
      if (!(byte & 128)) {
        if (value >= (1n << BigInt(bits))) return fail('malformed');
        return value;
      }
    }
    return fail('malformed');
  }
  // Preserve Wasm32 collection lengths and the reference's refusal ordering.
  length(max: number): number { const n = Number(this.varint(32)); if (n > max) return fail('limit'); return n; }
  raw(length: number): Uint8Array {
    if (length > this.bytes.length - this.offset) return fail('malformed');
    const span = this.bytes.subarray(this.offset, this.offset + length); this.offset += length; return span;
  }
}
function write(schema: Schema, value: unknown, out: Writer, budget: Budget, depth: number, types: Types): void {
  budget.visit(depth);
  const s = resolve(schema, types);
  const child = (item: Schema, value: unknown) => write(item, value, out, budget, depth + 1, types);
  switch (s.k) {
    case 'unit': if (value !== null) fail('value'); return;
    case 'bool': if (typeof value !== 'boolean') fail('value'); out.byte(value ? 1 : 0); return;
    case 'u': case 'i': {
      if (s.bits <= 32 ? typeof value !== 'number' || !Number.isSafeInteger(value) : typeof value !== 'bigint') fail('value');
      const n = BigInt(value as number | bigint), bits = BigInt(s.bits), signed = s.k === 'i';
      const min = signed ? -(1n << (bits - 1n)) : 0n, max = (1n << (bits - (signed ? 1n : 0n))) - 1n;
      if (n < min || n > max) fail('limit');
      if (s.bits === 8) out.byte(Number(n & 255n));
      else out.varint(signed ? (n << 1n) ^ (n >> (bits - 1n)) : n);
      return;
    }
    case 'bytes': {
      if (!(value instanceof Uint8Array)) return fail('value');
      if (s.min !== undefined && value.length < s.min) fail('value');
      if (s.max !== undefined && value.length > s.max) fail('limit');
      out.varint(BigInt(value.length)); out.raw(value); return;
    }
    case 'packed-i16': {
      if (!(value instanceof Int16Array)) return fail('value');
      if (value.byteLength / 2 > s.max) fail('limit');
      out.varint(BigInt(value.byteLength));
      if (value.byteLength > out.limit - out.offset) fail('limit');
      const span = new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
      if (littleEndian) out.raw(span);
      else { const copy = new Uint8Array(span); swap16(copy); out.raw(copy); }
      return;
    }
    case 'text': {
      if (typeof value !== 'string') return fail('value');
      const remaining = Math.min(s.max ?? out.limit, out.limit - out.offset);
      if (value.length > remaining) fail('limit');
      const buffer = new Uint8Array(Math.min(remaining, value.length * 3)), encoded = utf16.encodeInto(value, buffer);
      if (encoded.read !== value.length) fail('limit');
      const span = buffer.subarray(0, encoded.written);
      if (utf8.decode(span) !== value) fail('value');
      out.varint(BigInt(span.length)); out.raw(span); return;
    }
    case 'newtype': child(s.item, value); return;
    case 'list': {
      if (!Array.isArray(value)) return fail('value');
      if (value.length > (s.max ?? budget.remaining)) fail('limit');
      budget.length(value.length); out.varint(BigInt(value.length));
      for (let i = 0; i < value.length; i++) {
        if (!Object.hasOwn(value, i)) fail('value');
        child(s.item, value[i]);
      }
      return;
    }
    case 'option': {
      if (value === null) { out.byte(0); return; }
      const record = object(value, ['some']); out.byte(1); child(s.item, record.some); return;
    }
    case 'record': {
      const record = object(value, s.fields);
      for (const field of s.fields) child(field.value, record[field.name]);
      return;
    }
    case 'enum': {
      const record = object(value, ['tag', 'value']), variant = s.variants.find(v => v.name === record.tag);
      if (!variant) return fail('value');
      out.varint(BigInt(variant.id)); child(variant.value, record.value); return;
    }
    default: return fail('value');
  }
}
function read(schema: Schema, input: Reader, budget: Budget, depth: number, types: Types): unknown {
  // Reference decode resolves aliases before charging the concrete value.
  const s = resolve(schema, types);
  budget.visit(depth);
  const child = (item: Schema) => read(item, input, budget, depth + 1, types);
  switch (s.k) {
    case 'unit': return null;
    case 'bool': { const byte = input.byte(); if (byte > 1) return fail('malformed'); return byte === 1; }
    case 'u': case 'i': {
      let n = s.bits === 8 ? BigInt(input.byte()) : input.varint(s.bits);
      if (s.k === 'i') n = s.bits === 8 ? (n >= 128n ? n - 256n : n) : (n >> 1n) ^ -(n & 1n);
      return s.bits <= 32 ? Number(n) : n;
    }
    case 'bytes': {
      const length = input.length(s.max ?? input.bytes.length - input.offset);
      if (s.min !== undefined && length < s.min) fail('malformed');
      return new Uint8Array(input.raw(length));
    }
    case 'packed-i16': {
      const length = input.length(Math.min(s.max * 2, input.bytes.length - input.offset));
      if (length % 2) return fail('malformed');
      if (length === 0) return new Int16Array(0);
      const bytes = new Uint8Array(input.raw(length));
      if (!littleEndian) swap16(bytes);
      return new Int16Array(bytes.buffer);
    }
    case 'text': return utf8.decode(input.raw(input.length(Math.min(s.max ?? input.bytes.length, input.bytes.length - input.offset))));
    case 'newtype': return child(s.item);
    case 'list': {
      const length = input.length(Math.min(s.max ?? budget.remaining, budget.remaining));
      budget.length(length);
      const values: unknown[] = [];
      for (let i = 0; i < length; i++) values.push(child(s.item));
      return values;
    }
    case 'option': {
      const tag = input.byte(); if (tag === 0) return null; if (tag !== 1) return fail('malformed');
      return { some: child(s.item) };
    }
    case 'record': {
      const value = Object.create(null) as Record<string, unknown>;
      for (const field of s.fields) value[field.name] = child(field.value);
      return value;
    }
    case 'enum': {
      const id = Number(input.varint(32)), variant = s.variants.find(v => v.id === id);
      if (!variant) return fail('malformed');
      return { tag: variant.name, value: child(variant.value) };
    }
    default: return fail('value');
  }
}
function decodeError(error: unknown): never { if (error instanceof WireError) throw error; return fail('malformed'); }

// Factories consume generated private metadata. They do not validate arbitrary
// external schemas or clone/protect objects supplied by a custom trusted caller.
export function createPayloadCodec(metadata: unknown, registry: unknown = {}) {
  const meta = metadata as Payload, types = registry as Types;
  if (!Number.isSafeInteger(meta.bytes) || meta.bytes < 0) fail('value');
  for (const value of [meta.nodes, meta.depth]) if (!Number.isSafeInteger(value) || value < 1) fail('value');
  return {
    encode(value: unknown): Uint8Array {
      const out = new Writer(meta.bytes);
      write(meta.schema, value, out, new Budget(meta.nodes, meta.depth), 1, types);
      return out.finish();
    },
    decode(bytes: Uint8Array): unknown {
      if (!(bytes instanceof Uint8Array)) return fail('value');
      if (bytes.length > meta.bytes) return fail('limit');
      try {
        const input = new Reader(bytes);
        const value = read(meta.schema, input, new Budget(meta.nodes, meta.depth), 1, types);
        if (input.offset !== bytes.length) return fail('malformed');
        return value;
      } catch (error) { return decodeError(error); }
    }
  };
}
export function createCodec(metadata: unknown, registry: unknown = {}) {
  const meta = metadata as Method, types = registry as Types, id = BigInt(meta.id);
  if (id < 0n || id >= (1n << 64n)) fail('value');
  for (const value of [meta.requestBytes, meta.responseBytes, meta.requestNodes, meta.responseNodes, meta.requestDepth, meta.responseDepth]) {
    if (!Number.isSafeInteger(value) || value < 1) fail('value');
  }
  if (meta.requestBytes < 11 || meta.responseBytes < 11) fail('value');
  function encode(value: unknown, direction: 0 | 1): Uint8Array {
    const out = new Writer(direction ? meta.responseBytes : meta.requestBytes, 11);
    out.header(id, direction);
    write(direction ? meta.response : meta.request, value, out, new Budget(direction ? meta.responseNodes : meta.requestNodes, direction ? meta.responseDepth : meta.requestDepth), 1, types);
    return out.finish();
  }
  function decode(bytes: Uint8Array, direction: 0 | 1): unknown {
    if (!(bytes instanceof Uint8Array)) return fail('value');
    if (bytes.length > (direction ? meta.responseBytes : meta.requestBytes)) return fail('limit');
    if (bytes.length < 11) return fail('malformed');
    if (bytes[0] !== 73 || bytes[1] !== 87 || bytes[10] !== direction) return fail('identity');
    try {
      const input = new Reader(bytes, 11);
      if (input.view.getBigUint64(2, true) !== id) return fail('identity');
      const value = read(direction ? meta.response : meta.request, input, new Budget(direction ? meta.responseNodes : meta.requestNodes, direction ? meta.responseDepth : meta.requestDepth), 1, types);
      if (input.offset !== bytes.length) return fail('malformed');
      return value;
    } catch (error) { return decodeError(error); }
  }
  return { encodeRequest: (value: unknown) => encode(value, 0), decodeRequest: (bytes: Uint8Array) => decode(bytes, 0), encodeResponse: (value: unknown) => encode(value, 1), decodeResponse: (bytes: Uint8Array) => decode(bytes, 1) };
}

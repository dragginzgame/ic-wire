// Canonical bounded Postcard subset. Schemas come from trusted Rust declarations.
export class WireError extends Error {
  constructor(readonly code: 'limit' | 'malformed' | 'identity' | 'value') { super(`wire ${code}`); }
}
type Entry = { name: string; id: number; value: Schema };
type Schema = { k: 'unit' | 'bool' }
  | { k: 'u' | 'i'; bits: number }
  | { k: 'float'; bits: 32 | 64 }
  | { k: 'bytes' | 'text'; max: number }
  | { k: 'list' | 'array'; max: number; item: Schema }
  | { k: 'option'; item: Schema }
  | { k: 'result'; ok: Schema; err: Schema }
  | { k: 'record'; fields: readonly Entry[] }
  | { k: 'enum'; variants: readonly Entry[] };
type Metadata = { id: string; request: Schema; response: Schema; requestBytes: number; responseBytes: number; requestNodes: number; responseNodes: number; requestDepth: number; responseDepth: number };
const fail = (code: WireError['code']): never => { throw new WireError(code); };
const utf8 = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
const textEncoder = new TextEncoder();
function object(value: unknown, keys: readonly string[]): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return fail('value');
  const own = Reflect.ownKeys(value);
  if (own.length !== keys.length || keys.some(k => !Object.hasOwn(value, k))) return fail('value');
  return value as Record<string, unknown>;
}
class Budget {
  constructor(public nodes: number, readonly depth: number) {}
  visit(depth: number) { if (--this.nodes < 0 || depth > this.depth) fail('limit'); }
}
class Writer {
  readonly bytes: Uint8Array;
  offset = 11;
  constructor(size: number, id: bigint, direction: number) {
    this.bytes = new Uint8Array(size);
    this.bytes.set([73, 87]); new DataView(this.bytes.buffer).setBigUint64(2, id, true); this.bytes[10] = direction;
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
function write(s: Schema, value: unknown, out: Writer, budget: Budget, depth: number): void {
  budget.visit(depth);
  switch (s.k) {
    case 'unit': if (value !== null) fail('value'); return;
    case 'bool': if (typeof value !== 'boolean') fail('value'); out.byte(value ? 1 : 0); return;
    case 'float': {
      if (typeof value !== 'number' || !Number.isFinite(value)) fail('value');
      const n = value as number;
      if (s.bits === 32 && !Object.is(Math.fround(n), n)) fail('value');
      const bytes = new Uint8Array(s.bits / 8), view = new DataView(bytes.buffer);
      if (s.bits === 32) view.setFloat32(0, n, true); else view.setFloat64(0, n, true);
      out.raw(bytes); return;
    }
    case 'u': case 'i': {
      if (s.bits <= 32 ? typeof value !== 'number' || !Number.isSafeInteger(value) : typeof value !== 'bigint') fail('value');
      const n = BigInt(value as number | bigint), bits = BigInt(s.bits), signed = s.k === 'i';
      const min = signed ? -(1n << (bits - 1n)) : 0n, max = (1n << (bits - (signed ? 1n : 0n))) - 1n;
      if (n < min || n > max) fail('limit');
      if (s.bits === 8) { out.byte(Number(n & 255n)); return; }
      out.varint(signed ? (n << 1n) ^ (n >> (bits - 1n)) : n); return;
    }
    case 'bytes': {
      if (!(value instanceof Uint8Array)) fail('value');
      const bytes = value as Uint8Array;
      if (bytes.length > s.max) fail('limit'); out.varint(BigInt(bytes.length)); out.raw(bytes); return;
    }
    case 'text': {
      if (typeof value !== 'string') fail('value');
      const text = value as string;
      if (text.length > s.max) fail('limit');
      const buffer = new Uint8Array(s.max), result = textEncoder.encodeInto(text, buffer);
      if (result.read !== text.length) fail('limit');
      const bytes = buffer.subarray(0, result.written);
      if (utf8.decode(bytes) !== text) fail('value'); // Reject lone UTF-16 surrogates.
      out.varint(BigInt(bytes.length)); out.raw(bytes); return;
    }
    case 'list': case 'array': {
      if (!Array.isArray(value)) fail('value');
      const values = value as unknown[];
      if (s.k === 'array' ? values.length !== s.max : values.length > s.max) fail('limit');
      if (s.k === 'list') out.varint(BigInt(values.length));
      for (const item of values) write(s.item, item, out, budget, depth + 1); return;
    }
    case 'option': {
      if (value === null) { out.byte(0); return; }
      const record = object(value, ['some']); out.byte(1); write(s.item, record.some, out, budget, depth + 1); return;
    }
    case 'result': {
      const key = value && typeof value === 'object' && Object.hasOwn(value, 'ok') ? 'ok' : 'err';
      const record = object(value, [key]); out.byte(key === 'ok' ? 0 : 1); write(s[key], record[key], out, budget, depth + 1); return;
    }
    case 'record': {
      const record = object(value, s.fields.map(f => f.name));
      for (const field of s.fields) write(field.value, record[field.name], out, budget, depth + 1); return;
    }
    case 'enum': {
      const record = object(value, ['tag', 'value']);
      const variant = s.variants.find(v => v.name === record.tag); if (!variant) return fail('value');
      out.varint(BigInt(variant.id)); write(variant.value, record.value, out, budget, depth + 1); return;
    }
  }
}
function read(s: Schema, input: Reader, budget: Budget, depth: number): unknown {
  budget.visit(depth);
  switch (s.k) {
    case 'unit': return null;
    case 'bool': { const n = input.byte(); if (n > 1) return fail('malformed'); return n === 1; }
    case 'float': {
      const bytes = input.raw(s.bits / 8);
      const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
      const value = s.bits === 32 ? view.getFloat32(0, true) : view.getFloat64(0, true);
      if (!Number.isFinite(value)) return fail('malformed');
      return value;
    }
    case 'u': case 'i': {
      let n = s.bits === 8 ? BigInt(input.byte()) : input.varint(s.bits);
      if (s.k === 'i') n = s.bits === 8 ? (n >= 128n ? n - 256n : n) : (n >> 1n) ^ -(n & 1n);
      return s.bits <= 32 ? Number(n) : n;
    }
    case 'bytes': return input.raw(input.length(s.max)).slice();
    case 'text': return utf8.decode(input.raw(input.length(s.max)));
    case 'list': case 'array': {
      const length = s.k === 'list' ? input.length(s.max) : s.max;
      const values: unknown[] = [];
      for (let i = 0; i < length; i++) values.push(read(s.item, input, budget, depth + 1));
      return values;
    }
    case 'option': {
      const tag = input.byte(); if (tag === 0) return null; if (tag !== 1) return fail('malformed');
      return { some: read(s.item, input, budget, depth + 1) };
    }
    case 'result': {
      const tag = input.byte(); if (tag > 1) return fail('malformed');
      return tag === 0 ? { ok: read(s.ok, input, budget, depth + 1) } : { err: read(s.err, input, budget, depth + 1) };
    }
    case 'record': {
      const value = Object.create(null) as Record<string, unknown>;
      for (const field of s.fields) value[field.name] = read(field.value, input, budget, depth + 1);
      return value;
    }
    case 'enum': {
      const id = input.varint(32), variant = s.variants.find(v => BigInt(v.id) === id);
      if (!variant) return fail('malformed');
      return { tag: variant.name, value: read(variant.value, input, budget, depth + 1) };
    }
  }
}
export function createCodec(meta: Metadata) {
  const id = BigInt(meta.id);
  function encode(value: unknown, direction: 0 | 1): Uint8Array {
    const out = new Writer(direction ? meta.responseBytes : meta.requestBytes, id, direction);
    write(direction ? meta.response : meta.request, value, out, new Budget(direction ? meta.responseNodes : meta.requestNodes, direction ? meta.responseDepth : meta.requestDepth), 1);
    return out.bytes.slice(0, out.offset);
  }
  function decode(bytes: Uint8Array, direction: 0 | 1): unknown {
    if (!(bytes instanceof Uint8Array)) return fail('value');
    if (bytes.length > (direction ? meta.responseBytes : meta.requestBytes)) return fail('limit');
    if (bytes.length < 11) return fail('malformed');
    if (bytes[0] !== 73 || bytes[1] !== 87 || bytes[10] !== direction || new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getBigUint64(2, true) !== id) return fail('identity');
    try {
      const input = new Reader(bytes);
      const value = read(direction ? meta.response : meta.request, input, new Budget(direction ? meta.responseNodes : meta.requestNodes, direction ? meta.responseDepth : meta.requestDepth), 1);
      if (input.offset !== bytes.length) return fail('malformed');
      return value;
    } catch (error) { if (error instanceof WireError) throw error; return fail('malformed'); }
  }
  return { encodeRequest: (value: unknown) => encode(value, 0), decodeRequest: (bytes: Uint8Array) => decode(bytes, 0), encodeResponse: (value: unknown) => encode(value, 1), decodeResponse: (bytes: Uint8Array) => decode(bytes, 1) };
}

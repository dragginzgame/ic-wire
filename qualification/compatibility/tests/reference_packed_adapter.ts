// Test-only lowering for the legacy oracle's bounded-i16 schema spelling.
// Its interpreter remains unchanged; this is not shipped with generated clients.
import * as reference from './reference-runtime.ts';
export * from './reference-runtime.ts';
function lower(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(lower);
  if (value === null || typeof value !== 'object') return value;
  const result = Object.fromEntries(Object.entries(value).map(([key, child]) => [key, lower(child)]));
  if (result.k === 'packed' && result.word === 'i16' && Object.hasOwn(result, 'max')) {
    result.k = 'packed-i16';
    delete result.word;
  }
  return result;
}
export function createPayloadCodec(metadata: unknown, registry: unknown = {}, checks = {}) {
  return reference.createPayloadCodec(lower(metadata), lower(registry), checks);
}
export function createCodec(metadata: unknown, registry: unknown = {}, checks = {}) {
  return reference.createCodec(lower(metadata), lower(registry), checks);
}

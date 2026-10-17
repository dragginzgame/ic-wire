#!/usr/bin/env node
// Decode with a trusted, generated local contract module. Never infer a schema.
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const [modulePath, direction, hex, ...extra] = process.argv.slice(2);
try {
  if (!modulePath || !['request', 'response'].includes(direction) || !hex || extra.length || !/^(?:[0-9a-fA-F]{2})+$/.test(hex)) {
    throw new Error('usage: node scripts/dev/inspect-wire.mjs GENERATED.js request|response HEX');
  }
  const codec = await import(pathToFileURL(resolve(modulePath)).href);
  const decode = direction === 'request' ? codec.decodeRequest : codec.decodeResponse;
  if (typeof decode !== 'function') throw new Error('module must export the generated decodeRequest/decodeResponse functions');
  const value = decode(Uint8Array.from(Buffer.from(hex, 'hex')));
  const readable = (_, value) => typeof value === 'bigint' ? { $bigint: value.toString() }
    : value instanceof Uint8Array ? { $bytes: Buffer.from(value).toString('hex') }
    : Object.is(value, -0) ? { $float: '-0' } : value;
  console.log(JSON.stringify({ method: codec.methodName, direction, value }, readable, 2));
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}

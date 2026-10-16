import type { HttpAgent } from '@icp-sdk/core/agent';
import { serialize, deserialize } from '../generated/wire-bindings/index.js';
import contract from '../generated/contract.json';

export interface Request { account: bigint; revision: bigint; payload: Uint8Array }
export interface Response extends Request {}
export class WireError extends Error {
  constructor(readonly code: 'limit' | 'malformed' | 'identity') { super(code); }
}
const CONTRACT = contract.id;
const MAX_WIRE = contract.maxWire;
const MAX_BLOB = contract.maxBlob;
const MAX_U64 = (1n << 64n) - 1n;

export function encodeRequest(value: Request): Uint8Array {
  for (const n of [value.account, value.revision]) {
    if (typeof n !== 'bigint' || n < 0n || n > MAX_U64) throw new WireError('limit');
  }
  if (!(value.payload instanceof Uint8Array) || value.payload.length > MAX_BLOB) throw new WireError('limit');
  return serialize('Message', { contract: CONTRACT, direction: 0, account: value.account,
    revision: value.revision, payload: [Array.from(value.payload)] });
}

// Schema-specific preflight: no allocation from untrusted counts. This does not
// turn the generator's generic decoder into a bounded codec for arbitrary types.
export function decodeResponse(bytes: Uint8Array): Response {
  if (bytes.length > MAX_WIRE) throw new WireError('limit');
  let offset = 0;
  const take = (): number => {
    if (offset === bytes.length) throw new WireError('malformed');
    return bytes[offset++];
  };
  const varint = (bits: number): bigint => {
    let value = 0n;
    for (let i = 0; i < Math.ceil(bits / 7); i++) {
      const byte = take();
      value |= BigInt(byte & 127) << BigInt(i * 7);
      if (!(byte & 128)) {
        if (value >= (1n << BigInt(bits)) || (i > 0 && byte === 0)) throw new WireError('malformed');
        return value;
      }
    }
    throw new WireError('malformed');
  };
  if (varint(32) !== BigInt(CONTRACT) || take() !== 1) throw new WireError('identity');
  varint(64); varint(64);
  const length = varint(32);
  if (length > BigInt(MAX_BLOB)) throw new WireError('limit');
  if (BigInt(bytes.length - offset) !== length) throw new WireError('malformed');
  try {
    const decoded = deserialize('Message', bytes);
    if (decoded.bytes.length !== 0) throw new WireError('malformed');
    const value = decoded.value;
    const canonical = serialize('Message', value);
    if (canonical.length !== bytes.length || canonical.some((v, i) => v !== bytes[i])) throw new WireError('malformed');
    return { account: BigInt(value.account), revision: BigInt(value.revision), payload: Uint8Array.from(value.payload[0]) };
  } catch (error) {
    if (error instanceof WireError) throw error;
    throw new WireError('malformed');
  }
}

export function createClient(agent: HttpAgent, canisterId: string) {
  return {
    async evaluate(request: Request): Promise<Response> {
      const result = await agent.update(canisterId, { methodName: 'evaluate', arg: encodeRequest(request) });
      // Agent.update owns signatures, polling and certificate verification.
      return decodeResponse(result.reply);
    },
  };
}

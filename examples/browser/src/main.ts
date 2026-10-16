import { HttpAgent } from '@icp-sdk/core/agent';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { createClient, decodeResponse, encodeRequest, WireError } from './client.js';

interface Connection { host: string; canisterId: string }
interface Vector { account: string; revision: string; payload: number[]; request: number[]; response: number[] }
function assert(ok: unknown, message: string): asserts ok { if (!ok) throw new Error(message); }
function equal(a: Uint8Array, b: number[] | Uint8Array): boolean { return a.length === b.length && a.every((v, i) => v === b[i]); }
function rejected(action: () => unknown, code?: string): void {
  try { action(); } catch (error) {
    assert(error instanceof WireError, 'expected typed wire error');
    if (code) assert(error.code === code, `expected ${code}, got ${error.code}`);
    return;
  }
  throw new Error('expected rejection');
}

export async function run(connection: Connection, vectors: Vector[]) {
  const host = new URL(connection.host);
  assert(host.protocol === 'http:' && ['127.0.0.1', 'localhost'].includes(host.hostname), 'local fixture only');
  const identity = Ed25519KeyIdentity.generate();
  const agent = await HttpAgent.create({ host: host.href, identity, shouldFetchRootKey: true });
  const client = createClient(agent, connection.canisterId);
  let truncations = 0;
  for (const vector of vectors) {
    const request = { account: BigInt(vector.account), revision: BigInt(vector.revision), payload: Uint8Array.from(vector.payload) };
    assert(equal(encodeRequest(request), vector.request), 'TypeScript encode must equal Rust bytes');
    const expected = decodeResponse(Uint8Array.from(vector.response));
    assert(expected.account === request.account, 'bigint account');
    assert(expected.revision === ((request.revision + 1n) & ((1n << 64n) - 1n)), 'bigint revision');
    assert(equal(expected.payload, request.payload), 'payload round trip');
    for (let end = 0; end < vector.response.length; end++) {
      rejected(() => decodeResponse(Uint8Array.from(vector.response.slice(0, end)))); truncations++;
    }
    const response = await client.evaluate(request);
    assert(response.account === expected.account && response.revision === expected.revision, 'installed update result');
    assert(equal(response.payload, expected.payload), 'installed update payload');
  }
  const small = Uint8Array.from(vectors[0].response);
  rejected(() => decodeResponse(new Uint8Array(5121)), 'limit');
  rejected(() => decodeResponse(Uint8Array.from([...small, 0])), 'malformed');
  const wrongId = small.slice(); wrongId[0] ^= 1;
  rejected(() => decodeResponse(wrongId), 'identity');
  const wrongDirection = small.slice(); wrongDirection[3] = 0;
  rejected(() => decodeResponse(wrongDirection), 'identity');
  rejected(() => decodeResponse(Uint8Array.from([...small.slice(0, 6), 0x81, 0x20])), 'limit');
  rejected(() => decodeResponse(Uint8Array.from([...small.slice(0, 4), 0x87, 0, ...small.slice(5)])), 'malformed');
  rejected(() => encodeRequest({ account: -1n, revision: 0n, payload: new Uint8Array() }), 'limit');
  rejected(() => encodeRequest({ account: 1n << 64n, revision: 0n, payload: new Uint8Array() }), 'limit');
  rejected(() => encodeRequest({ account: 0n, revision: 0n, payload: new Uint8Array(4097) }), 'limit');
  // Check the real adapter's reject path through the signed agent transport.
  let canisterRejected = false;
  try { await agent.update(connection.canisterId, { methodName: 'evaluate', arg: new Uint8Array(5121) }); }
  catch (error) { canisterRejected = /wire limit/.test(String(error)); }
  assert(canisterRejected, 'canister must reject oversized input');
  // Mainnet root key deliberately cannot validate this local subnet. No verifier
  // overrides, disabled certificate checks, or hand-written polling are used.
  const wrongRoot = await HttpAgent.create({ host: host.href, identity, shouldFetchRootKey: false });
  let certificateRejected = false;
  try { await createClient(wrongRoot, connection.canisterId).evaluate({ account: 7n, revision: 42n, payload: new Uint8Array() }); }
  catch (error) {
    certificateRejected = /certificate|signature|verify|verification/i.test(String(error));
  }
  assert(certificateRejected, 'wrong root key must fail verification');
  const result = { vectors: vectors.length, signedUpdates: vectors.length, truncatedRepliesRejected: truncations,
    canisterRejected, certificateRejected, identity: identity.getPrincipal().toText(),
    bigintBoundary: '0, 2^53+1, u64::MAX (revision wraps)', selectedEndpoint: 'evaluate' };
  document.querySelector('output')!.textContent = JSON.stringify(result, null, 2);
  return result;
}

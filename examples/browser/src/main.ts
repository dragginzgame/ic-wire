import { Principal } from '@icp-sdk/core/principal';
import { HttpAgent } from '@icp-sdk/core/agent';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { createClient } from './client.js';
import * as apply from '../generated/apply.js';
import * as inspect from '../generated/inspect.js';
import { WireError } from '../generated/wire-runtime.js';

interface Connection { host: string; canisterId: string; senderId: string; caller: string; unavailableId: string }
interface Vectors { query: number[]; queryReply: number[]; request: number[]; success: number[]; failure: number[] }
function assert(ok: unknown, message: string): asserts ok { if (!ok) throw new Error(message); }
function equal(a: Uint8Array, b: number[] | Uint8Array): boolean { return a.length === b.length && a.every((v, i) => v === b[i]); }
async function rejected(action: () => unknown, pattern: RegExp): Promise<void> {
  try { await action(); } catch (error) { assert(pattern.test(String(error)), `unexpected rejection: ${error}`); return; }
  throw new Error('expected rejection');
}
export async function run(connection: Connection, vectors: Vectors) {
  const host = new URL(connection.host);
  assert(host.protocol === 'http:' && ['127.0.0.1', 'localhost'].includes(host.hostname), 'local fixture only');
  // Public deterministic seed belongs only to this disposable local test.
  const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(7));
  assert(identity.getPrincipal().toText() === connection.caller, 'fixture controller matches browser identity');
  const agent = await HttpAgent.create({ host: host.href, identity, shouldFetchRootKey: true });
  const client = createClient(agent, connection.canisterId);
  const query: inspect.Request = { labels: ['é🦀', '\ufefftext'], correlation: { some: (1n << 64n)-1n } };
  assert(equal(inspect.encodeRequest(query), vectors.query), 'Rust query request bytes');
  const initial = await client.inspect(query);
  assert(equal(inspect.encodeResponse(initial), vectors.queryReply), 'Rust installed query response bytes');
  const command: apply.Request = { subject: identity.getPrincipal().toUint8Array(), audience: Principal.fromText(connection.canisterId).toUint8Array(), expiry: (1n<<64n)-1n, id: 1n, amount: 7n };
  const retained = apply.encodeRequest(command);
  assert(equal(retained, vectors.request), 'Rust command request bytes');
  const success = await client.apply(command);
  assert(equal(apply.encodeResponse(success), vectors.success), 'Rust installed success bytes');
  const replay = await agent.update(connection.canisterId, { methodName: apply.methodName, arg: retained });
  assert(equal(replay.reply, vectors.success), 'exact retained replay reply');
  const failure = await client.apply({ ...command, id: 2n, amount: (1n<<64n)-1n });
  assert('err' in failure && failure.err.tag === 'Overflow', 'typed application failure');
  assert(equal(apply.encodeResponse(failure), vectors.failure), 'Rust failure bytes');
  const failedReplay = await client.apply({ ...command, id: 2n, amount: (1n<<64n)-1n });
  assert(equal(apply.encodeResponse(failedReplay), vectors.failure), 'retained failure');
  await rejected(() => client.apply({ ...command, amount: 8n }), /Conflict/);
  await rejected(() => client.apply({ ...command, subject: new Uint8Array([4]) }), /Caller/);
  await rejected(() => client.apply({ ...command, audience: new Uint8Array([4]) }), /Audience/);
  await rejected(() => client.apply({ ...command, expiry: 0n }), /Expired/);
  const outsider = await HttpAgent.create({ host: host.href, identity: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(8)), shouldFetchRootKey: true });
  await rejected(() => createClient(outsider, connection.canisterId).apply(command), /controller required/);
  await rejected(() => agent.update(connection.canisterId, { methodName: apply.methodName, arg: new Uint8Array(129) }), /request limit/);
  await rejected(() => agent.update(connection.canisterId, { methodName: apply.methodName, arg: retained.slice(0,-1) }), /Codec/);
  await rejected(() => agent.query(connection.canisterId, { methodName: inspect.methodName, arg: new Uint8Array(257) }).then(r => { if(r.status !== 'replied') throw new Error(r.reject_message); }), /request limit/);
  assert((await client.inspect(query)).balance === 7n, 'rejected commands leave state unchanged');
  const forwarded = apply.encodeRequest({ ...command, subject: Principal.fromText(connection.senderId).toUint8Array(), id: 3n, amount: 5n });
  const peer = await agent.update(connection.senderId, { methodName: 'runtime_forward', arg: forwarded });
  const receipt = apply.decodeResponse(peer.reply);
  assert('ok' in receipt && receipt.ok.id === 3n && receipt.ok.balance === 12n, 'peer typed receipt');
  assert((await client.inspect(query)).balance === 12n, 'two peer deliveries apply one effect');
  await rejected(() => agent.update(connection.senderId, { methodName: 'runtime_forward', arg: retained }), /invalid forward request/);
  await rejected(() => agent.update(connection.senderId, { methodName: 'runtime_forward', arg: new Uint8Array(129) }), /request limit/);
  await rejected(() => outsider.update(connection.senderId, { methodName: 'runtime_forward', arg: forwarded }), /controller required/);
  await rejected(() => agent.update(connection.senderId, { methodName: 'runtime_forward', arg: apply.encodeRequest({ ...command, subject: Principal.fromText(connection.senderId).toUint8Array(), audience: Principal.fromText(connection.unavailableId).toUint8Array() }) }), /transport failure/);
  let truncations = 0;
  for (const [decode, bytes] of [[inspect.decodeResponse, vectors.queryReply], [apply.decodeResponse, vectors.success], [apply.decodeResponse, vectors.failure]] as const) {
    for (let end = 0; end < bytes.length; end++) {
      try { decode(Uint8Array.from(bytes.slice(0,end))); throw new Error('accepted truncation'); }
      catch (error) { assert(error instanceof WireError, 'typed truncation rejection'); truncations++; }
    }
    await rejected(() => decode(Uint8Array.from([...bytes,0])), /wire/);
    const wrong = Uint8Array.from(bytes); wrong[2] ^= 1;
    await rejected(() => decode(wrong), /identity/);
  }
  await rejected(() => inspect.encodeRequest({ labels: ['x'.repeat(33)], correlation: null }), /limit/);
  await rejected(() => inspect.encodeRequest({ labels: ['\ud800'], correlation: null }), /value/);
  // Replaying an existing request prevents the negative certificate test creating an effect.
  const wrongRoot = await HttpAgent.create({ host: host.href, identity, shouldFetchRootKey: false });
  await rejected(() => createClient(wrongRoot, connection.canisterId).apply(command), /certificate|signature|verify|verification/i);
  assert((await client.inspect(query)).balance === 12n, 'final balance');
  const result = { runtimeEnvelope: true, rustVectors: 5, queryVerified: true, successReplayVerified: true, failureReplayVerified: true, authRejectionsVerified: true, invalidInputNoMutation: true, peerDeliveries: 2, peerEffects: 1, peerTransportFailureRejected: true, certificateRejected: true, truncatedRepliesRejected: truncations, finalBalance: '12', identity: connection.caller, selectedEndpoints: [inspect.methodName, apply.methodName, 'runtime_forward'] };
  document.querySelector('output')!.textContent = JSON.stringify(result, null, 2);
  return result;
}

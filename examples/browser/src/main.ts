import { HttpAgent } from '@icp-sdk/core/agent';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { createClient } from './client.js';
import * as apply from '../generated/apply.js';
import * as inspect from '../generated/inspect.js';
import { createCodec, WireError } from '../generated/wire-runtime.js';

interface Connection { host: string; canisterId: string; caller: string }
interface Vectors { query: number[]; queryReply: number[]; request: number[]; success: number[]; failure: number[] }
function assert(ok: unknown, message: string): asserts ok { if (!ok) throw new Error(message); }
function equal(a: Uint8Array, b: number[] | Uint8Array): boolean { return a.length === b.length && a.every((v, i) => v === b[i]); }
async function rejected(action: () => unknown, pattern: RegExp): Promise<void> {
  try { await action(); } catch (error) { assert(pattern.test(String(error)), `unexpected rejection: ${error}`); return; }
  throw new Error('expected rejection');
}
export async function run(connection: Connection, vectors: Vectors, nativeVectors: {id: string; request: number[]; response: number[]}) {
  const host = new URL(connection.host);
  assert(host.protocol === 'http:' && ['127.0.0.1', 'localhost'].includes(host.hostname), 'local fixture only');
  // Public deterministic seed belongs only to this disposable local test.
  const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(7));
  assert(identity.getPrincipal().toText() === connection.caller, 'fixture controller matches browser identity');
  const agent = await HttpAgent.create({ host: host.href, identity, shouldFetchRootKey: true });
  const client = createClient(agent, connection.canisterId);
  const tree = {tag:'Branch',value:[{tag:'Leaf',value:['é',(1n<<128n)-1n]}]};
  const native = createCodec({id:nativeVectors.id,request:{k:'ref',name:'Tree'},response:{k:'ref',name:'Tree'},requestBytes:4096,responseBytes:4096,requestNodes:4096,responseNodes:4096,requestDepth:64,responseDepth:64}, {Tree:{k:'enum',variants:[{name:'Leaf',id:0,value:{k:'tuple',items:[{k:'text'},{k:'u',bits:128}]}},{name:'Branch',id:1,value:{k:'list',item:{k:'ref',name:'Tree'}}}]}});
  assert(equal(native.encodeRequest(tree), nativeVectors.request), 'native recursive Rust request');
  const nativeResult = await agent.query(connection.canisterId, {methodName:'native_echo',arg:native.encodeRequest(tree)});
  assert(nativeResult.status === 'replied', 'native query replied');
  const nativeReply = new Uint8Array(nativeResult.reply.arg);
  assert(equal(nativeReply,nativeVectors.response), 'native recursive Rust reply');
  assert(equal(native.encodeResponse(native.decodeResponse(nativeReply)),nativeReply), 'native canonical browser roundtrip');
  const query: inspect.Request = { labels: ['é🦀', '\ufefftext'], correlation: { some: (1n << 64n)-1n } };
  assert(equal(inspect.encodeRequest(query), vectors.query), 'Rust query request bytes');
  const initial = await client.inspect(query);
  assert(equal(inspect.encodeResponse(initial), vectors.queryReply), 'Rust installed query response bytes');
  const command: apply.Request = { id: 1n, amount: 7n };
  const retained = apply.encodeRequest(command);
  assert(equal(retained, vectors.request), 'Rust command request bytes');
  const success = await client.apply(command);
  assert(equal(apply.encodeResponse(success), vectors.success), 'Rust installed success bytes');
  const failure = await client.apply({ ...command, id: 2n, amount: (1n<<64n)-1n });
  assert('err' in failure && failure.err.tag === 'Overflow', 'typed application failure');
  assert(equal(apply.encodeResponse(failure), vectors.failure), 'Rust failure bytes');
  const outsider = await HttpAgent.create({ host: host.href, identity: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(8)), shouldFetchRootKey: true });
  await rejected(() => createClient(outsider, connection.canisterId).apply(command), /controller required/);
  await rejected(() => agent.update(connection.canisterId, { methodName: apply.methodName, arg: new Uint8Array(129) }), /request limit/);
  await rejected(() => agent.update(connection.canisterId, { methodName: apply.methodName, arg: retained.slice(0,-1) }), /codec/);
  await rejected(() => agent.query(connection.canisterId, { methodName: inspect.methodName, arg: new Uint8Array(257) }).then(r => { if(r.status !== 'replied') throw new Error(r.reject_message); }), /request limit/);
  assert((await client.inspect(query)).balance === 7n, 'rejected commands leave state unchanged');
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
  // A zero increment keeps the certificate rejection probe free of state changes.
  const wrongRoot = await HttpAgent.create({ host: host.href, identity, shouldFetchRootKey: false });
  await rejected(() => createClient(wrongRoot, connection.canisterId).apply({id: 99n, amount: 0n}), /certificate|signature|verify|verification/i);
  assert((await client.inspect(query)).balance === 7n, 'final balance');
  const result = { runtimeEnvelope: true, nativeRecursiveQueryVerified: true, rustVectors: 5, queryVerified: true, successVerified: true, applicationFailureVerified: true, authRejectionsVerified: true, invalidInputNoMutation: true, certificateRejected: true, truncatedRepliesRejected: truncations, finalBalance: '7', identity: connection.caller, selectedEndpoints: [inspect.methodName, apply.methodName, "native_echo"] };
  document.querySelector('output')!.textContent = JSON.stringify(result, null, 2);
  return result;
}

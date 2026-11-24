import assert from 'node:assert/strict';
import { HttpAgent } from '@icp-sdk/core/agent';
import { agentTransport } from './agent-transport.ts';
import { createService } from '../target/service/service.ts';
import { add } from '../target/service/methods.ts';

const [host, id] = process.argv.slice(2);
assert(['localhost', '127.0.0.1'].includes(new URL(host).hostname));
const agent = await HttpAgent.create({ host, shouldFetchRootKey: true });
const transport = agentTransport(agent);
const counter = createService(id, transport);
assert.equal(await counter.read(0), 0);
assert.equal(await counter.add(128), 128);
assert.equal(await counter.read(1), 129);

const valid = add.encodeRequest(1);
const wrongId = valid.slice();
wrongId[2] ^= 1;
const wrongDirection = valid.slice();
wrongDirection[10] = 1;
for (const [frame, error] of [[valid.subarray(0, 11), /wire malformed/], [wrongId, /wire identity/], [wrongDirection, /wire identity/]]) {
  await assert.rejects(transport.update(id, 'add', frame), error);
}
assert.equal(await counter.read(0), 128);
console.log('Raw service query, update, and three canister rejections passed');

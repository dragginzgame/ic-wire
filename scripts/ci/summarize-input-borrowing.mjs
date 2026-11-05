// Audit physical-call receipts before summarizing; never infer missing trace costs.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const dir = process.argv[2];
const read = name => JSON.parse(fs.readFileSync(path.join(dir, name), 'utf8'));
const calls = fs.readFileSync(path.join(dir, 'calls.jsonl'), 'utf8').trim().split('\n').map(JSON.parse);
assert.equal(calls.length, 192);
const stats = values => ({ mean: values.reduce((a,b)=>a+b,0)/values.length, min: Math.min(...values), max: Math.max(...values), samples:values });
const cohorts = new Map();
let server;
const sequences=[];
const receipts=[];
for (const lane of ['owned','borrowed']) {
  const trace = read(`${lane}-executions.json`);
  assert.equal(trace.more,false); assert.equal(trace.truncated,false);
  assert.equal(BigInt(trace.dropped_executions),0n);
  assert.equal(BigInt(trace.dropped_instructions),0n);
  assert.equal(trace.dropped_instructions_incomplete,false);
  if (server) assert.equal(trace.server_binary_hash,server); else server=trace.server_binary_hash;
  const rows = calls.filter(row=>row.lane===lane);
  assert.equal(trace.executions.length,rows.length);
  const wasmHash=crypto.createHash('sha256').update(fs.readFileSync(path.join(dir,`${lane}.wasm`))).digest('hex');
  rows.forEach((row,i)=>{
    const execution=trace.executions[i];
    const sequence=Number(execution.sequence_number); sequences.push(sequence);
    if(i>0) assert.ok(sequence>Number(trace.executions[i-1].sequence_number));
    assert.equal(execution.module_hash,wasmHash);
    assert.equal(execution.function,'canister_update consume');
    assert.equal(execution.outcome,'reply'); assert.equal(execution.disposition.state,'committed');
    assert.equal(execution.attempt.instructions_available,true);
    assert.equal(execution.detail_truncated,false); assert.equal(execution.dirty_pages_tracked,true);
    receipts.push({call:row,execution});
    const key=`${row.size}/${row.mode}`;
    if(!cohorts.has(key)) cohorts.set(key,{size:row.size,mode:row.mode,owned:[],borrowed:[]});
    cohorts.get(key)[lane].push({...row,instructions:Number(execution.instructions),heap_page_instructions:Number(execution.page_access_instructions),cycles:Number(row.cycles)});
  });
}
assert.equal(new Set(sequences).size,192);
const results=[...cohorts.values()].map(group=>{
  for(let i=0;i<8;i++) {
    assert.equal(group.owned[i].request_sha256,group.borrowed[i].request_sha256);
    assert.equal(group.owned[i].reply_sha256,group.borrowed[i].reply_sha256);
    assert.equal(group.owned[i].repetition,i); assert.equal(group.borrowed[i].repetition,i);
  }
  const result={size:group.size,mode:['full','prefix256','reject'][group.mode],request_bytes:group.owned[0].request_bytes};
  for(const metric of ['instructions','heap_page_instructions','cycles']) {
    const owned=stats(group.owned.map(x=>x[metric])), borrowed=stats(group.borrowed.map(x=>x[metric]));
    result[metric]={owned,borrowed,change_percent:100*(borrowed.mean/owned.mean-1)};
  }
  return result;
});
fs.writeFileSync(path.join(dir,'comparison.json'),JSON.stringify({server_binary_hash:server,physical_updates:192,excluded_warmups:2,results},null,2)+'\n');
fs.writeFileSync(path.join(dir,'samples.jsonl'),receipts.map(x=>JSON.stringify(x)).join('\n')+'\n');
console.table(results.map(x=>({size:x.size,mode:x.mode,owned:x.instructions.owned.mean,borrowed:x.instructions.borrowed.mean,change:x.instructions.change_percent.toFixed(2)+'%',ownedPages:x.heap_page_instructions.owned.mean,borrowedPages:x.heap_page_instructions.borrowed.mean})));

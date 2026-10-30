import fs from 'node:fs';
import crypto from 'node:crypto';
import zlib from 'node:zlib';
import assert from 'node:assert/strict';
const root=process.argv[2], lanes=['wire_owned','wire_borrowed','wire_direct','wire_selective'];
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const stats=v=>({mean:v.reduce((a,b)=>a+b,0)/v.length,min:Math.min(...v),max:Math.max(...v),samples:v});
const raw=[];let server;
for(const profile of ['evaluation','benchmark-speed']) {
 const dir=`${root}/${profile}`,calls=fs.readFileSync(`${dir}/calls.jsonl`,'utf8').trim().split('\n').map(JSON.parse);
 assert.equal(calls.length,768);
 const groups=new Map(),artifacts=[],sequences=[];
 for(const lane of lanes) {
  const wasm=fs.readFileSync(`${dir}/${lane}.wasm`),moduleHash=hash(wasm);
  const exports=WebAssembly.Module.exports(new WebAssembly.Module(wasm)).filter(e=>e.name.startsWith('canister_')).map(e=>e.name).sort();
  assert.deepEqual(exports,['canister_update consume','canister_update encode']);
  artifacts.push({lane,exports,total_bytes:wasm.length,code_bytes:Number(fs.readFileSync(`${dir}/${lane}-code-bytes.txt`,'utf8')),sha256:moduleHash});
  const trace=JSON.parse(fs.readFileSync(`${dir}/${lane}-executions.json`));
  assert.equal(trace.more,false);assert.equal(trace.truncated,false);assert.equal(BigInt(trace.dropped_executions),0n);assert.equal(BigInt(trace.dropped_instructions),0n);assert.equal(trace.dropped_instructions_incomplete,false);
  if(server)assert.equal(trace.server_binary_hash,server);else server=trace.server_binary_hash;
  const rows=calls.filter(r=>r.lane===lane);assert.equal(rows.length,192);assert.equal(trace.executions.length,rows.length);
  rows.forEach((r,i)=>{
   const e=trace.executions[i];sequences.push(e.sequence_number);
   assert.equal(e.module_hash,moduleHash);assert.equal(e.function,`canister_update ${r.method}`);
   assert.equal(e.outcome,r.outcome==='reply'?'reply':'error');assert.equal(e.disposition.state,r.outcome==='reply'?'committed':'discarded');
   assert.equal(e.attempt.instructions_available,true);assert.equal(e.detail_truncated,false);
   if(!groups.has(r.case))groups.set(r.case,{case:r.case,size:r.size,mode:r.mode,outcome:r.outcome});
   const g=groups.get(r.case);g[lane]??=[];
   g[lane].push({...r,instructions:Number(e.instructions),pages:Number(e.page_access_instructions),cycles:Number(r.cycles)});
   raw.push({profile,call:r,execution:e});
  });
 }
 assert.equal(new Set(sequences).size,768);
 const results=[...groups.values()].map(g=>{
  for(const lane of lanes){assert.equal(g[lane].length,8);assert.deepEqual(g[lane].map(r=>r.repetition),[0,1,2,3,4,5,6,7]);}
  for(let i=0;i<8;i++)for(const lane of lanes)for(const f of ['request_sha256','reply_sha256','request_bytes','reply_bytes','outcome'])assert.deepEqual(g[lane][i][f],g.wire_owned[i][f]);
  return {case:g.case,size:g.size,mode:g.mode,outcome:g.outcome,lanes:Object.fromEntries(lanes.map(l=>[l,{instructions:stats(g[l].map(r=>r.instructions)),page_instructions:stats(g[l].map(r=>r.pages)),cycles:stats(g[l].map(r=>r.cycles)),request_bytes:[...new Set(g[l].map(r=>r.request_bytes))],reply_bytes:[...new Set(g[l].map(r=>r.reply_bytes))]}]))};
 });
 assert.equal(results.length,24);
 fs.writeFileSync(`${root}/${profile}.json`,JSON.stringify({profile,server,measured_calls:768,artifacts,results},null,2)+'\n');
 console.log(profile,JSON.stringify(artifacts));
}
fs.writeFileSync(`${root}/samples.json.gz`,zlib.gzipSync(JSON.stringify(raw)));

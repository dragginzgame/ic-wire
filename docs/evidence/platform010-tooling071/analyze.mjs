import fs from 'node:fs';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import zlib from 'node:zlib';
const [root, output] = process.argv.slice(2);
assert(root && output, 'study root and output directory required');
const expectedServer = fs.readFileSync(`${root}/server.sha256`, 'utf8').trim().split(/\s+/)[0];
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const read = path => JSON.parse(fs.readFileSync(path, 'utf8'));
const stats = samples => ({mean: samples.reduce((a,b)=>a+b,0)/samples.length, min:Math.min(...samples), max:Math.max(...samples), samples});
const delta = (before,after) => ({before,after,delta:after-before,percent:100*(after-before)/before});
function codeSize(wasm) {
  let p=8;
  while(p<wasm.length) {
    const id=wasm[p++]; let size=0,shift=0,byte;
    do {byte=wasm[p++];size+=(byte&127)*2**shift;shift+=7;} while(byte&128);
    if(id===10) return size;
    p+=size;
  }
  throw Error('missing code section');
}
const reports={}, raw={}; let logical=0,physical=0;
for(const profile of ['evaluation','benchmark-speed']) {
  const lanes={};
  for(const lane of ['baseline','candidate']) {
    const dir=`${root}/cost-${lane}-${profile}`;
    const wasm=fs.readFileSync(`${dir}/endpoint.wasm`), sha=hash(wasm);
    const rows=fs.readFileSync(`${dir}/samples.jsonl`,'utf8').trim().split('\n').map(JSON.parse);
    assert.equal(rows.length,108);
    for(const row of rows) {
      assert.equal(row.warmup,row.repetition===0);
      let instructions=0;
      for(const e of row.physical_executions) {
        assert.equal(e.module_hash,sha); assert.equal(e.attempt.instructions_available,true); assert.equal(e.detail_truncated,false);
        instructions+=Number(e.instructions);
      }
      assert.equal(instructions,row.ic_instructions);
      if(!row.warmup) {logical++;physical+=row.physical_executions.length;}
    }
    lanes[lane]={rows,wasm_bytes:wasm.length,code_bytes:codeSize(wasm),module_sha256:sha,server:read(`${dir}/trace-header.json`).server_binary_hash};
    assert.equal(lanes[lane].server, expectedServer);
  }
  const b=lanes.baseline,a=lanes.candidate;
  assert.equal(b.server,a.server);
  for(let i=0;i<b.rows.length;i++) for(const field of ['method','mode','case','repetition','warmup','request_sha256','request_bytes','reply_sha256','reply_bytes','outcome']) assert.deepEqual(b.rows[i][field],a.rows[i][field],`${profile}/${i}/${field}`);
  const cases=[];
  for(let i=0;i<a.rows.length;i+=9) {
    const br=b.rows.slice(i+1,i+9),ar=a.rows.slice(i+1,i+9),first=ar[0];
    assert.deepEqual(ar.map(r=>r.repetition),[1,2,3,4,5,6,7,8]);
    const before=stats(br.map(r=>r.ic_instructions)),after=stats(ar.map(r=>r.ic_instructions));
    cases.push({method:first.method,case:first.case,outcome:first.outcome,request_bytes:first.request_bytes,instructions:{...delta(before.mean,after.mean),before_samples:before,after_samples:after},cycles:delta(stats(br.map(r=>Number(r.cycles_debited))).mean,stats(ar.map(r=>Number(r.cycles_debited))).mean)});
  }
  reports[profile]={wasm:delta(b.wasm_bytes,a.wasm_bytes),code:delta(b.code_bytes,a.code_bytes),cases};raw[profile]=lanes;
}
fs.writeFileSync(`${output}/comparison.json`,JSON.stringify({logical,physical,reports},null,2)+'\n');
fs.writeFileSync(`${output}/samples.json.gz`,zlib.gzipSync(JSON.stringify(raw)));
console.log(JSON.stringify({logical,physical,profiles:Object.fromEntries(Object.entries(reports).map(([p,r])=>[p,{wasm:r.wasm,code:r.code,worst:r.cases.toSorted((a,b)=>b.instructions.percent-a.instructions.percent)[0]}]))},null,2));

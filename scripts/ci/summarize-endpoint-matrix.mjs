// Coverage is checked against the compiled exports, never a duplicate hand list.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const dir=process.argv[2];
const read=name=>fs.readFileSync(path.join(dir,name),'utf8');
const wasm=fs.readFileSync(path.join(dir,'endpoint.wasm'));
const moduleHash=crypto.createHash('sha256').update(wasm).digest('hex');
const exports=WebAssembly.Module.exports(new WebAssembly.Module(wasm))
  .filter(e=>/^canister_(query|composite_query|update) /.test(e.name)).map(e=>e.name).sort();
assert.ok(exports.length>0);
if(process.argv[3]==='--exports') {
  fs.writeFileSync(path.join(dir,'exports.json'),JSON.stringify(exports,null,2)+'\n');
} else {
  const rows=read('samples.jsonl').trim().split('\n').map(JSON.parse);
  const covered=[...new Set(rows.map(r=>`canister_${r.mode} ${r.method}`))].sort();
  assert.deepEqual(covered,exports);
  const cohorts=new Map();
  for(const row of rows) {
    const key=`${row.mode}/${row.method}/${row.case}`;
    if(!cohorts.has(key)) cohorts.set(key,[]);
    cohorts.get(key).push(row);
    assert.ok(row.physical_executions.length>0);
    let instructions=0, pages=0;
    for(const execution of row.physical_executions) {
      assert.equal(execution.module_hash,moduleHash);
      assert.equal(execution.attempt.instructions_available,true);
      assert.equal(execution.detail_truncated,false);
      instructions+=Number(execution.instructions); pages+=Number(execution.page_access_instructions);
    }
    assert.equal(instructions,row.ic_instructions); assert.equal(pages,row.heap_page_instructions);
  }
  const stats=values=>({mean:values.reduce((a,b)=>a+b,0)/values.length,min:Math.min(...values),max:Math.max(...values),samples:values});
  const results=[...cohorts.values()].map(group=>{
    assert.equal(group.length,9);
    assert.deepEqual(group.map(r=>r.repetition),[0,1,2,3,4,5,6,7,8]);
    assert.deepEqual(group.map(r=>r.warmup),[true,false,false,false,false,false,false,false,false]);
    const first=group[0], measured=group.slice(1);
    for(const r of measured) {
      assert.equal(r.request_sha256,first.request_sha256);
      assert.equal(r.request_bytes,first.request_bytes); assert.equal(r.outcome,first.outcome);
    }
    return {method:first.method,mode:first.mode,case:first.case,outcome:first.outcome,
      request_bytes:first.request_bytes,reply_bytes:[...new Set(measured.map(r=>r.reply_bytes))],
      ic_instructions:stats(measured.map(r=>r.ic_instructions)),
      heap_page_instructions:stats(measured.map(r=>r.heap_page_instructions)),
      cycles_debited:stats(measured.map(r=>{
        const value=Number(r.cycles_debited); assert.ok(Number.isSafeInteger(value)); return value;
      })),physical_executions:stats(measured.map(r=>r.physical_executions.length))};
  });
  const report={module_sha256:moduleHash,server_sha256:JSON.parse(read('trace-header.json')).server_binary_hash,
    total_wasm_bytes:wasm.length,code_section_bytes:Number(read('code-bytes.txt').trim()),
    exports,measured_calls:results.length*8,excluded_warmups:results.length,results};
  fs.writeFileSync(path.join(dir,'comparison.json'),JSON.stringify(report,null,2)+'\n');
  console.table(results.map(r=>({method:r.method,case:r.case,outcome:r.outcome,instructions:r.ic_instructions.mean,cycles:r.cycles_debited.mean,executions:r.physical_executions.mean})));
}

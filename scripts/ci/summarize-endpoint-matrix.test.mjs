import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

function fixture(t) {
  const scratch=new URL('../../target/',import.meta.url);
  fs.mkdirSync(scratch,{recursive:true});
  const dir=fs.mkdtempSync(new URL('endpoint-summary-',scratch));
  t.after(()=>fs.rmSync(dir,{recursive:true,force:true}));
  const name=Buffer.from('canister_query probe');
  const section=(id,bytes)=>[id,bytes.length,...bytes];
  const wasm=Buffer.from([0,97,115,109,1,0,0,0,
    ...section(1,[1,96,0,0]),...section(3,[1,0]),
    ...section(7,[1,name.length,...name,0,0]),...section(10,[1,2,0,11])]);
  const hash=crypto.createHash('sha256').update(wasm).digest('hex');
  fs.writeFileSync(path.join(dir,'endpoint.wasm'),wasm);
  fs.writeFileSync(path.join(dir,'code-bytes.txt'),'4\n');
  fs.writeFileSync(path.join(dir,'trace-header.json'),JSON.stringify({server_binary_hash:'fixture'}));
  const rows=Array.from({length:9},(_,repetition)=>({
    method:'probe',mode:'query',case:'unit',repetition,warmup:repetition===0,caller:`caller-${repetition}`,
    request_sha256:'fixture',request_bytes:12,reply_bytes:12,outcome:'reply',cycles_debited:'20',
    ic_instructions:100,heap_page_instructions:2,physical_executions:[{
      module_hash:hash,attempt:{instructions_available:true},detail_truncated:false,
      instructions:'100',page_access_instructions:'2',
    }],
  }));
  const run=(args=[])=>{
    fs.writeFileSync(path.join(dir,'samples.jsonl'),rows.map(row=>JSON.stringify(row)).join('\n')+'\n');
    return spawnSync(process.execPath,[new URL('./summarize-endpoint-matrix.mjs',import.meta.url).pathname,dir,...args],{encoding:'utf8'});
  };
  return {dir,rows,run};
}

test('complete evidence summarizes exact counters and compiled exports',t=>{
  const {dir,run}=fixture(t);
  const result=run();assert.equal(result.status,0,result.stderr);
  const report=JSON.parse(fs.readFileSync(path.join(dir,'comparison.json')));
  assert.equal(report.results[0].ic_instructions.mean,100);
  assert.equal(report.results[0].cycles_debited.mean,20);
  assert.equal(report.measured_calls,8);
  assert.equal(run(['--exports']).status,0);
  assert.deepEqual(JSON.parse(fs.readFileSync(path.join(dir,'exports.json'))),['canister_query probe']);
});

const corruptions={
  'duplicate caller':rows=>{rows[1].caller=rows[0].caller;},
  'missing caller':rows=>{delete rows[0].caller;},
  'unsafe execution counter':rows=>{rows[0].physical_executions[0].instructions='9007199254740992';},
  'negative counter':rows=>{rows[0].physical_executions[0].instructions='-1';},
  'fractional pages':rows=>{rows[0].physical_executions[0].page_access_instructions=1.5;},
  'null counter':rows=>{rows[0].physical_executions[0].instructions=null;},
  'empty counter':rows=>{rows[0].physical_executions[0].instructions='';},
  'unsafe row total':rows=>{rows[0].ic_instructions=9007199254740992;},
  'physical execution sum overflow':rows=>{
    rows[0].physical_executions[0].instructions=String(Number.MAX_SAFE_INTEGER);
    rows[0].physical_executions.push({...rows[0].physical_executions[0],instructions:'1'});
  },
  'negative cycles':rows=>{rows[0].cycles_debited='-1';},
  'unsafe cycles':rows=>{rows[0].cycles_debited='9007199254740992';},
  'aggregate overflow':rows=>{for(const row of rows) row.cycles_debited=String(Number.MAX_SAFE_INTEGER);},
  'mismatched module':rows=>{rows[0].physical_executions[0].module_hash='other';},
  'truncated detail':rows=>{rows[0].physical_executions[0].detail_truncated=true;},
};
for(const [name,corrupt] of Object.entries(corruptions)) test(`rejects ${name}`,t=>{
  const {dir,rows,run}=fixture(t);corrupt(rows);
  assert.notEqual(run().status,0);
  assert.equal(fs.existsSync(path.join(dir,'comparison.json')),false);
});
for(const bytes of ['-1','9007199254740992','1000','']) test(`rejects invalid code-section bytes ${JSON.stringify(bytes)}`,t=>{
  const {dir,run}=fixture(t);fs.writeFileSync(path.join(dir,'code-bytes.txt'),bytes);
  assert.notEqual(run().status,0);
});

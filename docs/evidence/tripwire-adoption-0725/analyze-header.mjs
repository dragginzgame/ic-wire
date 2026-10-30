import fs from 'node:fs';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import zlib from 'node:zlib';
const root=process.argv[2], candidate=process.argv[3]??'candidate-isolated';
const read=p=>JSON.parse(fs.readFileSync(p,'utf8'));
const jsonl=p=>fs.readFileSync(p,'utf8').trim().split('\n').map(JSON.parse);
const key=r=>[r.mode,r.method,r.case].join('/');
const change=(before,after)=>({before,after,delta:after-before,percent:before?100*(after-before)/before:null});
const reports={},raw={};
for(const profile of ['evaluation','benchmark-speed']) {
 const lanes={};
 for(const lane of ['baseline','candidate']) {
  const dir=`${root}/${lane==='candidate'?candidate:lane}/${profile}`;
  const report=read(`${dir}/comparison.json`),wasm=fs.readFileSync(`${dir}/endpoint.wasm`);
  assert.equal(crypto.createHash('sha256').update(wasm).digest('hex'),report.module_sha256);
  lanes[lane]={report,rows:jsonl(`${dir}/samples.jsonl`)};
 }
 const b=lanes.baseline,a=lanes.candidate;
 assert.deepEqual(b.report.exports,a.report.exports);assert.equal(b.report.server_sha256,a.report.server_sha256);
 assert.equal(b.rows.length,a.rows.length);
 for(let i=0;i<b.rows.length;i++)for(const f of ['method','mode','case','repetition','warmup','request_sha256','request_bytes','reply_sha256','reply_bytes','outcome'])assert.deepEqual(b.rows[i][f],a.rows[i][f],`${profile}/${i}/${f}`);
 const before=new Map(b.report.results.map(r=>[key(r),r]));
 const rows=a.report.results.map(r=>{
  const old=before.get(key(r));assert(old);
  return {method:r.method,mode:r.mode,case:r.case,outcome:r.outcome,request_bytes:r.request_bytes,
   instructions:change(old.ic_instructions.mean,r.ic_instructions.mean),pages:change(old.heap_page_instructions.mean,r.heap_page_instructions.mean),cycles:change(old.cycles_debited.mean,r.cycles_debited.mean)};
 });
 assert.equal(rows.length,before.size);
 reports[profile]={exports:a.report.exports.length,cases:rows.length,measured_calls_per_lane:a.report.measured_calls,
  wasm:change(b.report.total_wasm_bytes,a.report.total_wasm_bytes),code:change(b.report.code_section_bytes,a.report.code_section_bytes),rows};
 raw[profile]=lanes;
}
fs.writeFileSync(`${root}/header-${candidate}.json`,JSON.stringify(reports,null,2)+'\n');
fs.writeFileSync(`${root}/header-${candidate}-samples.json.gz`,zlib.gzipSync(JSON.stringify(raw)));
for(const [p,r] of Object.entries(reports))console.log(p,JSON.stringify({wasm:r.wasm,code:r.code,largest_valid_regression:r.rows.filter(x=>x.outcome==='reply').sort((a,b)=>b.instructions.percent-a.instructions.percent)[0],largest_saving:r.rows.toSorted((a,b)=>a.instructions.delta-b.instructions.delta)[0]}));

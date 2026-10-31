// Evidence checks and uncompressed artifact metrics; no gzip-size objective.
import fs from 'node:fs';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import zlib from 'node:zlib';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const root=process.argv[2],here=fileURLToPath(new URL('.',import.meta.url));
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
function artifact(path) {
 const b=fs.readFileSync(path);let p=8,code;
 while(p<b.length){const id=b[p++];let n=0,s=0,x;do{x=b[p++];n+=(x&127)*2**s;s+=7;}while(x&128);if(id===10)code=n;p+=n;}
 assert(code>0);
 return {total:b.length,code,sha256:hash(b)};
}
const scale=[];
for(const profile of ['evaluation','benchmark-speed'])for(const endpoints of [1,10,100]){
 const row={profile,endpoints};
 for(const lane of ['baseline','candidate','shared']){
  const path=`${root}/scale-${lane}-${endpoints}-${profile}.wasm`;
  const exports=WebAssembly.Module.exports(new WebAssembly.Module(fs.readFileSync(path))).filter(e=>e.name.startsWith('canister_update '));
  assert.equal(exports.length,endpoints);row[lane]=artifact(path);
 }scale.push(row);
}
fs.writeFileSync(`${root}/scale.json`,JSON.stringify(scale,null,2)+'\n');
// Reuse the existing physical-trace verifier unchanged, with directory aliases
// confined to the disposable study directory.
fs.mkdirSync(`${root}/input-results`,{recursive:true});
for(const profile of ['evaluation','benchmark-speed']){
 for(const lane of ['wire_owned','wire_borrowed','wire_direct','wire_selective']){
  const path=`${root}/input-${profile}/${lane}.wasm`;
  fs.writeFileSync(`${root}/input-${profile}/${lane}-code-bytes.txt`,`${artifact(path).code}\n`);
 }
 const link=`${root}/input-results/${profile}`;
 if(!fs.existsSync(link))fs.symlinkSync(`../input-${profile}`,link,'dir');
}
execFileSync(process.execPath,[`${here}../tripwire-adoption-0725/selective/analyze.mjs`,`${root}/input-results`],{stdio:'inherit'});
const smallInput=['evaluation','benchmark-speed'].map(profile=>({profile,
 before:artifact(`${root}/input-${profile}/wire_borrowed.wasm`),
 after:artifact(`${root}/stack-input-${profile}.wasm`)}));
fs.writeFileSync(`${root}/small-input-size.json`,JSON.stringify(smallInput,null,2)+'\n');
const dir=`${root}/compression-results`;
const rows=fs.readFileSync(`${dir}/samples.jsonl`,'utf8').trim().split('\n').map(JSON.parse);
assert.equal(rows.length,1368); // 19 inputs × 2 operations × 2 lanes × 2 profiles × 9.
const groups=new Map(),callers=new Set();
const artifacts={};
for(const profile of ['evaluation','benchmark-speed'])for(const lane of ['copy','lz4'])artifacts[`${profile}/${lane}`]=artifact(`${dir}/${profile}-${lane}.wasm`);
for(const row of rows){
 assert.equal(row.wasm_sha256,artifacts[`${row.profile}/${row.lane}`].sha256);
 assert.equal(row.warmup,row.repetition===0);assert(!callers.has(row.caller));callers.add(row.caller);
 const key=`${row.profile}/${row.lane}/${row.case}/${row.operation}`;
 if(!groups.has(key))groups.set(key,[]);groups.get(key).push(row);
}
const stats=s=>({mean:s.reduce((a,b)=>a+b,0)/s.length,min:Math.min(...s),max:Math.max(...s),samples:s});
assert.equal(groups.size,152);
const results=[...groups.values()].map(g=>{
 assert.deepEqual(g.map(r=>r.repetition),[0,1,2,3,4,5,6,7,8]);
 for(const r of g){assert.equal(r.input_sha256,g[0].input_sha256);assert.equal(r.output_sha256,g[0].output_sha256);}
 const {profile,lane,case:name,operation,input_sha256,output_sha256}=g[0];
 return {profile,lane,case:name,operation,input_sha256,output_sha256,body_instructions:stats(g.slice(1).map(r=>r.body_instructions))};
});
const ratios=fs.readFileSync(`${dir}/ratios.jsonl`,'utf8').trim().split('\n').map(JSON.parse);
fs.writeFileSync(`${dir}/summary.json`,JSON.stringify({artifacts,ratios:ratios.length,smaller:ratios.filter(r=>r.lz4_bytes<r.raw_bytes).length,measured_calls:results.length*8,results},null,2)+'\n');
fs.writeFileSync(`${dir}/samples.json.gz`,zlib.gzipSync(JSON.stringify(rows)));

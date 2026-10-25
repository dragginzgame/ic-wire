import fs from 'node:fs';
import {execFileSync} from 'node:child_process';
import crypto from 'node:crypto';
const root=process.argv[2];
if (!root) throw new Error('pass an absolute experiment directory containing source and parser-before.rs');
const source=`${root}/source`;
const before=fs.readFileSync(`${root}/parser-before.rs`,'utf8');
let after=before;
for (const width of [16,32,64]) {
 const needle='        let mut out = (first & 0x7f) as u'+width+';';
 after=after.replace(needle,
  '        self.take_varint_u'+width+'_tail(first)\n    }\n\n'+
  '    #[cfg_attr(target_arch = "wasm32", inline(never))]\n'+
  '    #[cfg_attr(not(target_arch = "wasm32"), inline(always))]\n'+
  '    fn take_varint_u'+width+'_tail(&mut self, first: u8) -> Result<u'+width+'> {\n'+needle);
}
fs.writeFileSync(`${root}/parser-candidate.rs`,after);
function run(...args) { try { return execFileSync(...args); } catch (e) { if (e.status === 0 && e.stdout) return e.stdout; throw e; } }
let rows=[];
for (const [cohort,features] of [['standard',''],['hundred','derive-scale']]) {
 for (const variant of ['baseline','candidate']) {
  fs.writeFileSync(`${source}/crates/ic-wire/src/native/parser/mod.rs`,variant==='baseline'?before:after);
  const out=`${root}/${cohort}-${variant}`;fs.mkdirSync(out,{recursive:true});
  for(const profile of ['evaluation','benchmark-speed']) {
   const build=run('cargo',['build','--offline','--locked','--profile',profile,'-p','ic-wire-benchmarks','--example','codec-wasm','--target','wasm32-unknown-unknown','--features',`wire${features?','+features:''}`,'--message-format=json'],{cwd:source,env:{...process.env,CARGO_TARGET_DIR:`${root}/target`},encoding:'utf8',maxBuffer:50e6});
   fs.writeFileSync(`${out}/${profile}-build.jsonl`,build);
   const artifact=build.trim().split('\n').map(l=>JSON.parse(l)).findLast(r=>r.reason==='compiler-artifact'&&r.target.name==='codec-wasm').filenames.find(f=>f.endsWith('.wasm'));
   const data=fs.readFileSync(artifact);fs.writeFileSync(`${out}/${profile}-wire.wasm`,data);
   let code=0;for(let p=8;p<data.length;){const kind=data[p++];let n=0,shift=0,b;do{b=data[p++];n+=(b&127)*2**shift;shift+=7;}while(b&128);if(kind===10)code=n;p+=n;}
   const row={cohort,variant,profile,total:data.length,code,sha256:crypto.createHash('sha256').update(data).digest('hex')};rows.push(row);console.log(row);fs.writeFileSync(`${root}/sizes.json`,JSON.stringify(rows,null,2)+'\n');
  }
 }
}
fs.writeFileSync(`${source}/crates/ic-wire/src/native/parser/mod.rs`,before);

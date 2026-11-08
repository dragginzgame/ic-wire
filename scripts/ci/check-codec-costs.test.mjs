import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { compareCosts } from './check-codec-costs.mjs';
const sha = bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
const profiles=['evaluation','benchmark-speed'];
function fixture(t) {
  const scratch=new URL('../../target/',import.meta.url);
  fs.mkdirSync(scratch,{recursive:true});
  const directory=fs.mkdtempSync(new URL('cost-gate-',scratch));
  t.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
  for(const lane of ['before','after']) {
    const root=path.join(directory,lane);fs.mkdirSync(path.join(root,'instructions'),{recursive:true});
    const identity={compiler:'rustc fixture',build_environment:{},counter:'ic0.performance_counter(0)',samples:3,
      server_version:'fixture 1',sample_caller:'unique callers',small_repetitions:4,large_repetitions:1,artifacts:[]};
    for(const field of ['workspace_manifest_sha256','lock_sha256','server_sha256','fixture_sha256',
      'hotspots_fixture_sha256','signed32_fixture_sha256','bulk_fixture_sha256','packed_i16_fixture_sha256','toko_fixture_sha256',
      'derive_fixture_sha256','byte_keys_fixture_sha256','cost_smoke_fixture_sha256','id16_fixture_sha256','shapes_fixture_sha256','driver_sha256','wasm_driver_sha256','evidence_helper_sha256','operations_sha256',
      'native_encoder_sha256','native_decoder_sha256','native_admission_sha256']) identity[field]=sha(field);
    let csv='profile,codec,wasm_bytes,wasm_sha256,code_section_bytes\n',call=0;const rows=[];
    for(const profile of profiles) for(const codec of ['wire','candid']) {
      const wasm=Buffer.from([0,97,115,109,1,0,0,0,10,1,0]);
      fs.writeFileSync(path.join(root,`${profile}-${codec}.wasm`),wasm);
      identity.artifacts.push({profile,codec,sha256:sha(wasm)});
      fs.writeFileSync(path.join(root,`${profile}-${codec}-build.jsonl`),[
        {reason:'compiler-artifact',target:{name:'ic_wire'},fresh:false},
        {reason:'compiler-artifact',target:{name:'codec-wasm'},features:[codec,'cost-smoke'],profile:{opt_level:profile==='evaluation'?'z':'3'},fresh:false},
      ].map(r=>JSON.stringify(r)).join('\n')+'\n');
      csv+=`${profile},${codec},${wasm.length},${sha(wasm)},1\n`;
      const callers=[++call,++call,++call];
      for(const operation of ['encode-fresh','decode-owned-or-labelled-borrowed','reject-bad-header','reject-truncated','loop-control'])
        rows.push({profile,case:'control',codec:codec==='wire'?'wire-native':codec,operation,
          encoded_bytes:12,input_sha256:sha('input'),payload_sha256:sha(codec),repetitions:4,
          samples_instructions:[4000,4000,4000],sample_caller_ids:callers,median_instructions_per_op:1000});
    }
    fs.writeFileSync(path.join(root,'wasm.csv'),csv);
    fs.writeFileSync(path.join(root,'instructions/identity.json'),JSON.stringify(identity));
    fs.writeFileSync(path.join(root,'instructions/samples.jsonl'),rows.map(r=>JSON.stringify(r)).join('\n')+'\n');
  }
  const before=path.join(directory,'before'),after=path.join(directory,'after');
  return {before,after,
    identity(change) {const file=path.join(after,'instructions/identity.json');const v=JSON.parse(fs.readFileSync(file));change(v);fs.writeFileSync(file,JSON.stringify(v));},
    rows(change,root=after) {const file=path.join(root,'instructions/samples.jsonl');let v=fs.readFileSync(file,'utf8').trim().split('\n').map(s=>JSON.parse(s));v=change(v)??v;fs.writeFileSync(file,v.map(r=>JSON.stringify(r)).join('\n')+'\n');}};
}
test('paired unchanged evidence passes with both raw Wasm metrics',t=>{
  const f=fixture(t),result=compareCosts(f.before,f.after);
  assert.deepEqual(result.failures,[]);assert.equal(result.instructions.length,8);
  assert(result.wasm.every(row=>row.total_delta===0&&row.code_delta===0));
});
for (const codec of ['wire-bounded','wire-borrowed','wire-packed-i16']) test(`qualified ${codec} rows remain comparable`,t=>{
  const f=fixture(t);
  for (const root of [f.before,f.after]) f.rows(rows=>{for(const row of rows) if(row.codec==='wire-native') row.codec=codec;},root);
  const result=compareCosts(f.before,f.after);
  assert.deepEqual(result.failures,[]);assert.equal(result.instructions.length,8);
});
for (const [name,change,expected] of [
  ['Candid-only evidence',rows=>rows.filter(r=>r.codec==='candid'),/missing Wire control/],
  ['unknown codec labels',rows=>rows.map(r=>r.codec==='wire-native'?{...r,codec:'misspelled-wire'}:r),/unsupported codec/],
  ['unknown operation labels',rows=>[...rows,...rows.filter(r=>r.operation==='loop-control').map(r=>({...r,operation:'unknown-operation'}))],/unsupported operation/],
]) test(`rejects matching ${name} instead of silently passing`,t=>{
  const f=fixture(t);
  for (const root of [f.before,f.after]) f.rows(change,root);
  assert.throws(()=>compareCosts(f.before,f.after),expected);
});
test('a Candid-only case cannot hide among valid paired cases',t=>{
  const f=fixture(t);
  for (const root of [f.before,f.after]) f.rows(rows=>[...rows,...rows.filter(r=>r.codec==='candid').map(r=>({
    ...r,case:'unpaired-control',sample_caller_ids:r.sample_caller_ids.map(id=>id+100),
  }))],root);
  assert.throws(()=>compareCosts(f.before,f.after),/missing Wire control/);
});
test('optional Wire reuse rows are checked without requiring a Candid reuse operation',t=>{
  const f=fixture(t);
  for (const root of [f.before,f.after]) f.rows(rows=>[...rows,...rows.filter(r=>r.codec==='wire-native'&&r.operation==='encode-fresh').map(r=>({...r,operation:'encode-reuse'}))],root);
  const result=compareCosts(f.before,f.after);
  assert.deepEqual(result.failures,[]);assert.equal(result.instructions.length,10);
});
test('instruction regression fails while the small absolute margin is explicit',t=>{
  const f=fixture(t);f.rows(rows=>{rows[0].samples_instructions=[4800,4800,4800];rows[0].median_instructions_per_op=1200;});
  assert.match(compareCosts(f.before,f.after).failures[0],/1000 → 1200 instructions/);
});
for(const [name,change,expected] of [
  ['dependency drift',f=>f.identity(v=>v.lock_sha256=sha('other lock')),/identity mismatch/],
  ['producer helper drift',f=>f.identity(v=>v.evidence_helper_sha256=sha('other producer')),/identity mismatch/],
  ['missing operation layout identity',f=>f.identity(v=>delete v.operations_sha256),/missing identity/],
  ['signed32 fixture drift',f=>f.identity(v=>v.signed32_fixture_sha256=sha('other signed fixture')),/identity mismatch/],
  ['ID fixture drift',f=>f.identity(v=>v.id16_fixture_sha256=sha('other ID fixture')),/identity mismatch/],
  ['nested fixture drift',f=>f.identity(v=>v.derive_fixture_sha256=sha('other scale fixture')),/identity mismatch/],
  ['Candid counter drift',f=>f.rows(rows=>{const r=rows.find(r=>r.codec==='candid');r.samples_instructions=[4004,4004,4004];r.median_instructions_per_op=1001;}),/Candid control changed/],
  ['missing compiler',f=>f.identity(v=>delete v.compiler),/missing identity/],
  ['changed input',f=>f.rows(rows=>{rows[0].input_sha256=sha('other input');}),/input/],
  ['missing operation',f=>f.rows(rows=>rows.slice(1)),/missing operation/],
  ['unstable counters',f=>f.rows(rows=>{rows[0].samples_instructions[0]++;}),/unstable/],
  ['cached runtime change',f=>{f.identity(v=>v.native_encoder_sha256=sha('changed source'));
    for(const profile of profiles) { const file=path.join(f.after,`${profile}-wire-build.jsonl`);fs.writeFileSync(file,fs.readFileSync(file,'utf8').replaceAll('\"fresh\":false','\"fresh\":true')); }},/cached library/],
  ['bad code-section metric',f=>{const file=path.join(f.after,'wasm.csv');fs.writeFileSync(file,fs.readFileSync(file,'utf8').replace(/,1\n/g,',2\n'));},/code-section size mismatch/],
  ['artifact mutation',f=>fs.appendFileSync(path.join(f.after,'evaluation-wire.wasm'),Buffer.from([0])),/checksum mismatch/],
]) test(name,t=>{const f=fixture(t);change(f);assert.throws(()=>compareCosts(f.before,f.after),expected);});

function changeWireModules(f,wasm,code) {
  const file=path.join(f.after,'wasm.csv');const lines=fs.readFileSync(file,'utf8').trim().split('\n');
  const updated=lines.map(line=>{
    const columns=line.split(',');if(columns[1]!=='wire') return line;
    const [profile,codec]=columns;
    fs.writeFileSync(path.join(f.after,`${profile}-${codec}.wasm`),wasm);
    return `${profile},${codec},${wasm.length},${sha(wasm)},${code}`;
  });
  fs.writeFileSync(file,updated.join('\n')+'\n');
  f.identity(v=>{for(const artifact of v.artifacts) if(artifact.codec==='wire') artifact.sha256=sha(wasm);});
}
test('total growth is caught even with unchanged code section',t=>{
  const f=fixture(t);changeWireModules(f,Buffer.from([0,97,115,109,1,0,0,0,10,1,0,0,1,0]),1);
  const result=compareCosts(f.before,f.after);
  assert(result.failures.some(s=>s.includes('total: 11 → 14 bytes')));
  assert(!result.failures.some(s=>s.includes('code:')));
});
test('code growth is an independently enforced limit',t=>{
  const f=fixture(t);changeWireModules(f,Buffer.from([0,97,115,109,1,0,0,0,1,4,1,96,0,0,3,2,1,0,10,4,1,2,0,11]),4);
  assert(compareCosts(f.before,f.after).failures.some(s=>s.includes('code: 1 → 4 bytes')));
});
test('changed runtime accepts an explicitly rebuilt candidate',t=>{
  const f=fixture(t);f.identity(v=>v.native_encoder_sha256=sha('new encoder'));
  assert.deepEqual(compareCosts(f.before,f.after).failures,[]);
});
test('compressed artifacts cannot masquerade as raw Wasm',async t=>{
  const {gzipSync}=await import('node:zlib');
  const f=fixture(t);changeWireModules(f,gzipSync(Buffer.from([0,97,115,109,1,0,0,0,10,1,0])),1);
  assert.throws(()=>compareCosts(f.before,f.after),/raw core Wasm/);
});
test('effective compiler profile overrides are rejected',t=>{
  const f=fixture(t),file=path.join(f.after,'evaluation-wire-build.jsonl');
  fs.writeFileSync(file,fs.readFileSync(file,'utf8').replace('"opt_level":"z"','"opt_level":"3"'));
  assert.throws(()=>compareCosts(f.before,f.after),/compiler profile mismatch/);
});
test('guest feature changes require a matched baseline',t=>{
  const f=fixture(t),file=path.join(f.after,'evaluation-wire-build.jsonl');
  fs.writeFileSync(file,fs.readFileSync(file,'utf8').replace('"cost-smoke"','"other-corpus"'));
  assert.throws(()=>compareCosts(f.before,f.after),/guest feature mismatch/);
});

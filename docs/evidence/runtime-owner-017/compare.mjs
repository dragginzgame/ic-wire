// Compare source-qualified owners; the maintained reporter audits physical traces.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
const root=process.argv[2];
assert.ok(root,'usage: node compare.mjs RAW_STUDY_DIRECTORY');
const read=file=>JSON.parse(fs.readFileSync(file,'utf8'));
const lines=file=>fs.readFileSync(file,'utf8').trim().split('\n').map(JSON.parse);
const rows=[],artifacts=[];
let samplePairs=0,identicalInstructionSamples=0,identicalCycleSamples=0;
for(const profile of ['evaluation','benchmark-speed']) {
  const dirs=['platform016','runtime017'].map(lane=>path.join(root,'costs',lane,profile));
  const reports=dirs.map(dir=>read(path.join(dir,'comparison.json')));
  assert.deepEqual(reports[0].exports,reports[1].exports);
  assert.equal(reports[0].server_sha256,reports[1].server_sha256);
  const samples=dirs.map(dir=>lines(path.join(dir,'samples.jsonl')));
  assert.equal(samples[0].length,samples[1].length);
  for(let i=0;i<samples[0].length;i++) {
    samplePairs++;
    identicalInstructionSamples+=Number(samples[0][i].ic_instructions===samples[1][i].ic_instructions);
    identicalCycleSamples+=Number(samples[0][i].cycles_debited===samples[1][i].cycles_debited);
    for(const field of ['caller','case','method','mode','repetition','warmup','request_sha256','request_bytes','reply_sha256','reply_bytes','outcome'])
      {
      assert.ok(Object.hasOwn(samples[0][i],field)&&Object.hasOwn(samples[1][i],field),field);
      assert.deepEqual(samples[0][i][field],samples[1][i][field],`${profile}/${i}/${field}`);
    }
  }
  artifacts.push({profile,exports:reports[0].exports.length,cases:reports[0].results.length,
    measured_calls_per_lane:reports[0].measured_calls,
    baseline:{total:reports[0].total_wasm_bytes,code:reports[0].code_section_bytes},
    candidate:{total:reports[1].total_wasm_bytes,code:reports[1].code_section_bytes}});
  assert.equal(reports[0].results.length,reports[1].results.length);
  for(let i=0;i<reports[0].results.length;i++) {
    const [a,b]=reports.map(r=>r.results[i]);
    for(const field of ['method','mode','case','outcome','request_bytes','reply_bytes'])assert.deepEqual(a[field],b[field]);
    rows.push({profile,method:a.method,mode:a.mode,case:a.case,outcome:a.outcome,
      request_bytes:a.request_bytes,
      baseline_instructions:a.ic_instructions.mean,candidate_instructions:b.ic_instructions.mean,
      instruction_delta:b.ic_instructions.mean-a.ic_instructions.mean,
      instruction_percent:(b.ic_instructions.mean/a.ic_instructions.mean-1)*100,
      baseline_cycles:a.cycles_debited.mean,candidate_cycles:b.cycles_debited.mean,
      baseline_page_instructions:a.heap_page_instructions.mean,candidate_page_instructions:b.heap_page_instructions.mean,
      baseline_samples:a.ic_instructions.samples,candidate_samples:b.ic_instructions.samples});
  }
}
const sorted=[...rows].sort((a,b)=>b.instruction_percent-a.instruction_percent);
const summary={qualification:'Frozen-source owner extraction; no registry publication claim',
  artifacts,pairs:rows.length,sample_pairs_including_warmup:samplePairs,
  identical_instruction_samples_including_warmup:identicalInstructionSamples,
  identical_cycle_samples_including_warmup:identicalCycleSamples,
  measured_calls:artifacts.reduce((n,a)=>n+2*a.measured_calls_per_lane,0),
  identical_instruction_pairs:rows.filter(r=>r.instruction_delta===0).length,
  identical_cycle_pairs:rows.filter(r=>r.baseline_cycles===r.candidate_cycles).length,
  worst:sorted[0],best:sorted.at(-1),
  instruction_regressions_over_five_percent:rows.filter(r=>r.instruction_percent>5),
  unit_and_blob_controls:rows.filter(r=>['echo','borrowed_echo','default_unit'].includes(r.method)&&['unit','blob-0','blob-32'].includes(r.case))};
fs.writeFileSync(path.join(root,'paired-summary.json'),JSON.stringify(summary,null,2)+'\n');
fs.writeFileSync(path.join(root,'paired-results.json'),JSON.stringify(rows,null,2)+'\n');
const columns=['profile','method','mode','case','outcome','request_bytes','baseline_instructions','candidate_instructions','instruction_delta','instruction_percent','baseline_cycles','candidate_cycles','baseline_page_instructions','candidate_page_instructions'];
const csv=value=>typeof value==='string'?'"'+value.replaceAll('"','""')+'"':String(value);
fs.writeFileSync(path.join(root,'paired-results.csv'),columns.join(',')+'\n'+rows.map(r=>columns.map(c=>csv(r[c])).join(',')).join('\n')+'\n');
console.log(JSON.stringify({artifacts,pairs:summary.pairs,measured_calls:summary.measured_calls,
  identical_instructions:summary.identical_instruction_pairs,identical_cycles:summary.identical_cycle_pairs,
  worst:summary.worst,best:summary.best,over_five_percent:summary.instruction_regressions_over_five_percent.length},null,2));

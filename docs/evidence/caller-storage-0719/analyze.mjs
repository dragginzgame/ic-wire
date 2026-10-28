import fs from 'node:fs';
import path from 'node:path';
const root = process.argv[2];
const read = name => JSON.parse(fs.readFileSync(path.join(root,name)));
const save = (name, header, rows) => fs.writeFileSync(path.join(root,name), [header.join(','),...rows.map(row=>row.join(','))].join('\n')+'\n');
const median = values => [...values].sort((a,b)=>a-b)[Math.floor(values.length/2)];
const change = (before,after) => ((after/before-1)*100).toFixed(2);
const instructions = read('instructions.json');
const ic = instructions.filter(x=>x.lane==='before-vec').map(before => {
  const find = lane => instructions.find(x=>x.profile===before.profile && x.case===before.case && x.lane===lane);
  const old=median(before.ic_instructions), vec=median(find('after-vec').ic_instructions), slice=median(find('after-slice').ic_instructions);
  return [before.profile,before.case,before.bytes,old,vec,slice,change(old,vec),change(old,slice)];
});
save('ic.csv',['profile','case','payload_bytes','before_vec_instructions','after_vec_instructions','slice_instructions','vec_change_pct','slice_change_pct'],ic);
const native=[];
for(const profile of ['evaluation','benchmark-speed']) {
  const rows=read(`${profile}-native/native-time.json`);
  for(const before of rows.filter(x=>x.lane==='vec-fresh')) {
    const time=lane=>{const row=rows.find(x=>x.case===before.case&&x.lane===lane); return median(row.samples.map(x=>x.native_ns/x.repetitions));};
    native.push([profile,before.case,before.bytes,time('vec-fresh'),time('vec-reuse'),time('slice'),change(time('vec-fresh'),time('slice'))]);
  }
}
save('native.csv',['profile','case','payload_bytes','vec_fresh_ns','vec_reuse_ns','slice_ns','slice_vs_fresh_pct'],native);
const runs=['results-shared-writer.json','results-shared-writer-2.json','results-shared-writer-3.json'].map(name=>read(`javascript/${name}`));
const javascript=[];
for(const before of runs[0].rows.filter(x=>x.lane==='before-framed')) for(const key of ['encode_ns','decode_ns']) {
  const time=lane=>median(runs.flatMap(run=>{const row=run.rows.find(x=>x.case===before.case&&x.lane===lane);return row[key].map(x=>x/row.repetitions);}));
  javascript.push([before.case,key,time('before-framed'),time('after-framed'),change(time('before-framed'),time('after-framed')),time('after-payload')]);
}
save('javascript.csv',['case','operation','before_framed_ns','after_framed_ns','framed_change_pct','payload_ns'],javascript);
const before=read('host-before/native-schema-host-benchmark.json'),after=read('host-after/native-schema-host-benchmark.json');
const generation=[];
for(const types of [1,100]) for(const derived of [false,true]) {
  const time=rows=>median(rows.filter(x=>x.types===types&&x.derived===derived).map(x=>x.ns_per_iteration));
  generation.push([types,derived,time(before),time(after),change(time(before),time(after))]);
}
save('host-generation.csv',['types','derived','before_ns','after_ns','change_pct'],generation);

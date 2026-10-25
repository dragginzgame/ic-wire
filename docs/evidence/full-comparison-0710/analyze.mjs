import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';
import assert from 'node:assert/strict';
const [root, output] = process.argv.slice(2);
assert(root && output, 'usage: node analyze.mjs RUN_ROOT OUTPUT');
fs.mkdirSync(output, { recursive: true });
const read = p => fs.readFileSync(p, 'utf8');
const jsonl = p => read(p).trim().split('\n').filter(Boolean).map(s => JSON.parse(s));
const key = r => [r.profile, r.case, r.codec, r.operation].join('|');
const csv = (name, rows) => {
  assert(rows.length);
  const columns = Object.keys(rows[0]);
  fs.writeFileSync(path.join(output, name), [columns.join(','), ...rows.map(r => columns.map(c => JSON.stringify(r[c] ?? '')).join(','))].join('\n') + '\n');
};
const cohorts = read(path.join(root, 'current/cohorts.psv')).trim().split('\n').map(s => s.split('|'));
const comparisons = [], rejections = [], all = [], artifacts = [], wins = [], regressions = [], raw = {};
let currentServer, currentLock;
for (const [cohort, features, primary] of cohorts) {
  const dir = path.join(root, cohort === 'hotspots' ? 'hotspots' : `current/${cohort}`);
  const instructions = jsonl(path.join(dir, 'instructions/samples.jsonl'));
  const index = new Map(instructions.map(r => [key(r), r]));
  assert.equal(index.size, instructions.length, `duplicate IC keys: ${cohort}`);
  const native = {}, nativeIdentities = {};
  for (const profile of ['evaluation', 'benchmark-speed']) for (const mode of ['native-1', 'native-2', 'allocations', 'heap']) {
    nativeIdentities[`${profile}-${mode}`] = JSON.parse(read(path.join(dir, `${profile}-${mode}/identity.json`)));
    native[`${profile}-${mode}`] = jsonl(path.join(dir, `${profile}-${mode}/samples.jsonl`));
    const idx = new Map(native[`${profile}-${mode}`].map(r => [key({...r, profile}), r]));
    assert.equal(idx.size, native[`${profile}-${mode}`].length);
    native[`${profile}-${mode}-index`] = idx;
  }
  raw[cohort] = {features, primary, identity: JSON.parse(read(path.join(dir, 'instructions/identity.json'))), instructions, nativeIdentities,
    native: Object.fromEntries(Object.entries(native).filter(([k]) => !k.endsWith('-index')))};
  currentServer ??= raw[cohort].identity.server_sha256;
  currentLock ??= raw[cohort].identity.lock_sha256;
  assert.equal(raw[cohort].identity.server_sha256, currentServer);
  assert.equal(raw[cohort].identity.lock_sha256, currentLock);
  for (const r of instructions) {
    assert.equal(r.samples_instructions.length, 3);
    all.push({cohort, ...r});
    if (r.codec === primary && ['reject-bad-header', 'reject-truncated'].includes(r.operation)) {
      const candid = index.get(key({...r, codec: 'candid'}));
      assert(candid, `missing rejection control ${cohort}/${key(r)}`);
      assert.equal(r.input_sha256, candid.input_sha256);
      rejections.push({cohort, profile:r.profile, case:r.case, operation:r.operation,
        wire_instructions:r.median_instructions_per_op, candid_instructions:candid.median_instructions_per_op,
        instruction_ratio_wire_over_candid:r.median_instructions_per_op/candid.median_instructions_per_op});
    }
    if (r.codec !== primary || !['encode-fresh', 'decode-owned-or-labelled-borrowed'].includes(r.operation)) continue;
    const candid = index.get(key({...r, codec: 'candid'}));
    assert(candid, `missing Candid ${cohort}/${key(r)}`);
    assert.equal(r.input_sha256, candid.input_sha256);
    const row = {cohort, profile:r.profile, case:r.case, operation:r.operation, wire_codec:r.codec,
      wire_bytes:r.encoded_bytes, candid_bytes:candid.encoded_bytes,
      wire_instructions:r.median_instructions_per_op, candid_instructions:candid.median_instructions_per_op,
      instruction_ratio_wire_over_candid:r.median_instructions_per_op / candid.median_instructions_per_op};
    for (const [label, lane] of [['wire',r],['candid',candid]]) for (const mode of ['native-1','native-2','allocations','heap']) {
      const n = native[`${r.profile}-${mode}-index`].get(key(lane));
      assert(n, `missing native ${cohort}/${mode}/${key(lane)}`);
      assert.equal(n.encoded_sha256, lane.payload_sha256);
      assert.equal(n.input_sha256, lane.input_sha256);
      if(mode.startsWith('native')) row[`${label}_${mode}_median_ns`] = n.measurement.median_ns;
      else for (const [k,v] of Object.entries(n.measurement)) if(typeof v === 'number') row[`${label}_${mode}_${k}`] = v;
    }
    comparisons.push(row);
  }
  const lines = read(path.join(dir, 'wasm.csv')).trim().split('\n');
  for (const line of lines.slice(1)) {
    const [profile, codec, bytes, sha256, code] = line.split(',');
    artifacts.push({cohort, profile, codec, total_bytes:+bytes, code_section_bytes:+code, sha256});
  }
  for (const profile of ['evaluation','benchmark-speed']) for (const operation of ['encode-fresh','decode-owned-or-labelled-borrowed']) {
    const rows = comparisons.filter(r => r.cohort === cohort && r.profile === profile && r.operation === operation);
    wins.push({cohort,profile,operation,cases:rows.length,wire_wins:rows.filter(r=>r.wire_instructions<r.candid_instructions).length,
      ties:rows.filter(r=>r.wire_instructions===r.candid_instructions).length,candid_wins:rows.filter(r=>r.wire_instructions>r.candid_instructions).length,
      max_wire_over_candid:Math.max(...rows.map(r=>r.instruction_ratio_wire_over_candid))});
  }
  for (const kind of ['baseline-results','release-baseline-results']) {
    const previous = path.join(root,kind,cohort,'instructions/samples.jsonl');
    if(!fs.existsSync(previous)) continue;
    const before = jsonl(previous);
    assert.equal(before.length, instructions.length);
    for(const b of before) {
      const a = index.get(key(b));
      assert(a, `missing current ${key(b)}`);
      assert.equal(b.input_sha256,a.input_sha256);
      assert.equal(b.payload_sha256,a.payload_sha256);
      regressions.push({baseline:kind,cohort,profile:a.profile,case:a.case,codec:a.codec,operation:a.operation,
        before:b.median_instructions_per_op,after:a.median_instructions_per_op,
        percent_change:100*(a.median_instructions_per_op/b.median_instructions_per_op-1)});
    }
    const identity = JSON.parse(read(path.join(root,kind,cohort,'instructions/identity.json')));
    for (const field of ['server_sha256', 'driver_sha256', 'wasm_driver_sha256', 'fixture_sha256', 'hotspots_fixture_sha256', 'bulk_fixture_sha256', 'packed_i16_fixture_sha256', 'toko_fixture_sha256', 'derive_fixture_sha256', 'samples', 'small_repetitions', 'large_repetitions']) {
      assert.equal(identity[field], raw[cohort].identity[field], `baseline identity ${cohort}/${kind}/${field}`);
    }
    raw[cohort][kind] = {instructions:before,identity};
  }
}
assert.equal(artifacts.length, cohorts.length * 4);
assert.equal(new Set(artifacts.map(r=>`${r.cohort}|${r.profile}|${r.codec}`)).size, artifacts.length);
const baselines = {};
for (const [kind, name] of [['baseline-results','normalized-artifacts.csv'],['release-baseline-results','release-artifacts.csv']]) {
  const source = path.join(root,kind,'comparison.csv');
  const lines = read(source).trim().split('\n');
  const columns = lines.shift().split(',');
  const rows = lines.map(line=>Object.fromEntries(line.split(',').map((value,index)=>[columns[index],value])));
  assert.equal(rows.length, artifacts.length);
  assert.equal(new Set(rows.map(r=>`${r.cohort}|${r.profile}|${r.codec}`)).size, rows.length);
  for (const r of rows) {
    const current = artifacts.find(a=>a.cohort===r.cohort&&a.profile===r.profile&&a.codec===r.codec);
    assert(current);
    assert.equal(r.after_sha256, current.sha256);
    assert.equal(+r.after_bytes, current.total_bytes);
    assert.equal(+r.after_code, current.code_section_bytes);
    if (r.identical === 'true') {
      assert.equal(r.before_sha256, r.after_sha256);
      assert.equal(r.before_bytes, r.after_bytes);
      assert.equal(r.before_code, r.after_code);
    } else {
      assert.equal(r.identical, 'false');
      assert(raw[r.cohort][kind], `differing artifact requires IC measurements: ${kind}/${r.cohort}`);
    }
  }
  fs.copyFileSync(source, path.join(output,name));
  baselines[kind] = {artifacts:rows.length, identical:rows.filter(r=>r.identical==='true').length,
    total_size_increases:rows.filter(r=>+r.after_bytes>+r.before_bytes).length,
    code_size_increases:rows.filter(r=>+r.after_code>+r.before_code).length,
    measured_instruction_rows:regressions.filter(r=>r.baseline===kind).length,
    instruction_increases:regressions.filter(r=>r.baseline===kind&&r.after>r.before).length};
}
csv('comparison.csv',comparisons);
csv('rejection-comparison.csv',rejections);
csv('artifacts.csv',artifacts);
csv('instruction-wins.csv',wins);
if(regressions.length) csv('instruction-regressions.csv',regressions);
fs.writeFileSync(path.join(output,'all-lanes.json.gz'),zlib.gzipSync(JSON.stringify(all)));
fs.writeFileSync(path.join(output,'raw.json.gz'),zlib.gzipSync(JSON.stringify(raw)));
const payloads = comparisons.filter(r=>r.profile==='benchmark-speed'&&r.operation==='encode-fresh');
const summary = {cohorts:cohorts.length,case_entries:comparisons.length/4,primary_comparisons:comparisons.length,
  baselines,
  payload_sizes:{smaller:payloads.filter(r=>r.wire_bytes<r.candid_bytes).length,
    equal:payloads.filter(r=>r.wire_bytes===r.candid_bytes).length,
    larger:payloads.filter(r=>r.wire_bytes>r.candid_bytes).length},
  all_instruction_rows:all.length, varying_instruction_rows:all.filter(r=>new Set(r.samples_instructions).size>1).length,
  wire_wins:comparisons.filter(r=>r.wire_instructions<r.candid_instructions).length,
  ties:comparisons.filter(r=>r.wire_instructions===r.candid_instructions).length,
  worst:comparisons.toSorted((a,b)=>b.instruction_ratio_wire_over_candid-a.instruction_ratio_wire_over_candid).slice(0,20),
  regressions:regressions.filter(r=>r.after>r.before).toSorted((a,b)=>b.percent_change-a.percent_change).slice(0,20)};
fs.writeFileSync(path.join(output,'summary.json'),JSON.stringify(summary,null,2)+'\n');
console.log(JSON.stringify({cohorts:summary.cohorts,cases:summary.case_entries,comparisons:comparisons.length,regression_rows:regressions.length}));

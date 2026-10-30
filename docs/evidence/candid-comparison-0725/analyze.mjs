import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';
import crypto from 'node:crypto';
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
const cohorts = read(path.join(root, 'cohorts.psv')).trim().split('\n').map(s => s.split('|'));
const comparisons = [], rejections = [], all = [], artifacts = [], wins = [], raw = {};
const samplingControls = {}, samplingChanges = [];
let currentServer, currentLock;
for (const [cohort, features, primary] of cohorts) {
  const dir = path.join(root, cohort);
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
  const controlPath = path.join(dir, 'instructions-cached-control');
  if (fs.existsSync(controlPath)) {
    const before = jsonl(path.join(controlPath, 'samples.jsonl'));
    const identity = JSON.parse(read(path.join(controlPath, 'identity.json')));
    assert.deepEqual(identity.artifacts, raw[cohort].identity.artifacts);
    assert.equal(identity.server_sha256, raw[cohort].identity.server_sha256);
    assert.equal(identity.lock_sha256, raw[cohort].identity.lock_sha256);
    assert.equal(before.length, instructions.length);
    for (const b of before) {
      const a = index.get(key(b));
      assert(a, `missing fresh sample ${cohort}/${key(b)}`);
      assert.equal(a.input_sha256, b.input_sha256);
      assert.equal(a.payload_sha256, b.payload_sha256);
      if (a.median_instructions_per_op !== b.median_instructions_per_op)
        samplingChanges.push({cohort, key:key(a), before:b.median_instructions_per_op, after:a.median_instructions_per_op});
    }
    samplingControls[cohort] = {identity, instructions:before};
  }
  currentServer ??= raw[cohort].identity.server_sha256;
  currentLock ??= raw[cohort].identity.lock_sha256;
  assert.equal(raw[cohort].identity.server_sha256, currentServer);
  assert.equal(raw[cohort].identity.lock_sha256, currentLock);
  assert.equal(raw[cohort].identity.sample_caller, 'unique 8-byte little-endian principal per query; caller not read by guest');
  for (const identity of Object.values(nativeIdentities)) {
    assert.equal(identity.lock_sha256, currentLock);
    assert.equal(identity.samples, 11);
    assert.equal(identity.minimum_batch_ms, 2);
    assert.equal(identity.case_count, nativeIdentities['evaluation-native-1'].case_count);
  }
  const callerGroups = new Map();
  for (const r of instructions) {
    assert.equal(r.samples_instructions.length, 3);
    assert.equal(r.sample_caller_ids.length, 3);
    assert(r.sample_caller_ids.every(id => Number.isSafeInteger(id) && id > 0));
    const group = [r.profile, r.case, r.codec].join('|');
    if (callerGroups.has(group)) assert.deepEqual(r.sample_caller_ids, callerGroups.get(group));
    else callerGroups.set(group, r.sample_caller_ids);
    const other = index.get(key({...r, profile: r.profile === 'evaluation' ? 'benchmark-speed' : 'evaluation'}));
    assert(other, `missing profile control: ${cohort}/${key(r)}`);
    assert.equal(r.input_sha256, other.input_sha256);
    assert.equal(r.payload_sha256, other.payload_sha256);
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
  assert.equal(new Set([...callerGroups.values()].flat()).size, 3 * callerGroups.size, `reused sample caller: ${cohort}`);
  const lines = read(path.join(dir, 'wasm.csv')).trim().split('\n');
  for (const line of lines.slice(1)) {
    const [profile, codec, bytes, sha256, code] = line.split(',');
    const wasm = fs.readFileSync(path.join(dir, `${profile}-${codec}.wasm`));
    assert.equal(wasm.length, +bytes);
    assert.equal(crypto.createHash('sha256').update(wasm).digest('hex'), sha256);
    assert.equal(raw[cohort].identity.artifacts.find(a => a.profile === profile && a.codec === codec)?.sha256, sha256);
    artifacts.push({cohort, profile, codec, total_bytes:+bytes, code_section_bytes:+code, sha256});
  }
  for (const profile of ['evaluation','benchmark-speed']) for (const operation of ['encode-fresh','decode-owned-or-labelled-borrowed']) {
    const rows = comparisons.filter(r => r.cohort === cohort && r.profile === profile && r.operation === operation);
    assert.equal(rows.length, nativeIdentities[`${profile}-native-1`].case_count, `incomplete primary coverage: ${cohort}/${profile}/${operation}`);
    wins.push({cohort,profile,operation,cases:rows.length,wire_wins:rows.filter(r=>r.wire_instructions<r.candid_instructions).length,
      ties:rows.filter(r=>r.wire_instructions===r.candid_instructions).length,candid_wins:rows.filter(r=>r.wire_instructions>r.candid_instructions).length,
      max_wire_over_candid:Math.max(...rows.map(r=>r.instruction_ratio_wire_over_candid))});
  }
}
csv('comparison.csv',comparisons);
csv('rejection-comparison.csv',rejections);
csv('artifacts.csv',artifacts);
csv('instruction-wins.csv',wins);
fs.writeFileSync(path.join(output,'all-lanes.json.gz'),zlib.gzipSync(JSON.stringify(all)));
fs.writeFileSync(path.join(output,'raw.json.gz'),zlib.gzipSync(JSON.stringify(raw)));
if (Object.keys(samplingControls).length) {
  assert.equal(Object.keys(samplingControls).length, cohorts.length);
  fs.writeFileSync(path.join(output,'sampling-controls.json.gz'),zlib.gzipSync(JSON.stringify(samplingControls)));
}
const payloads = comparisons.filter(r=>r.profile==='benchmark-speed'&&r.operation==='encode-fresh');
const summary = {cohorts:cohorts.length,case_entries:comparisons.length/4,primary_comparisons:comparisons.length,
  payload_sizes:{smaller:payloads.filter(r=>r.wire_bytes<r.candid_bytes).length,
    equal:payloads.filter(r=>r.wire_bytes===r.candid_bytes).length,
    larger:payloads.filter(r=>r.wire_bytes>r.candid_bytes).length},
  all_instruction_rows:all.length, varying_instruction_rows:all.filter(r=>new Set(r.samples_instructions).size>1).length,
  sampling_control_cohorts:Object.keys(samplingControls).length, sampling_changes:samplingChanges,
  wire_wins:comparisons.filter(r=>r.wire_instructions<r.candid_instructions).length,
  ties:comparisons.filter(r=>r.wire_instructions===r.candid_instructions).length,
  worst:comparisons.toSorted((a,b)=>b.instruction_ratio_wire_over_candid-a.instruction_ratio_wire_over_candid).slice(0,20)};
fs.writeFileSync(path.join(output,'summary.json'),JSON.stringify(summary,null,2)+'\n');
console.log(JSON.stringify({cohorts:summary.cohorts,cases:summary.case_entries,comparisons:comparisons.length}));

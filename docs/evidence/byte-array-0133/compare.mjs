// Compare the paired cohort outputs after analyze.mjs has checked their evidence.
import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';
import assert from 'node:assert/strict';

const [directory] = process.argv.slice(2);
assert(directory, 'usage: node compare.mjs ANALYZED_DIRECTORY');
const raw = JSON.parse(zlib.gunzipSync(fs.readFileSync(path.join(directory, 'raw.json.gz'))));
const before = raw['byte-keys-control'], after = raw['byte-keys'];
const key = r => [r.profile, r.case, r.codec, r.operation].join('|');
const index = rows => new Map(rows.map(r => [key(r), r]));
// The IC-only loop control has no corresponding native/allocator operation.
const measured = rows => rows.filter(r => r.operation !== 'loop-control');
const previous = index(measured(before.instructions));
const change = (a, b) => 100 * (b / a - 1);
const rows = [];
for (const row of measured(after.instructions)) {
  const old = previous.get(key(row));
  assert(old, `missing baseline: ${key(row)}`);
  assert.equal(old.input_sha256, row.input_sha256);
  const n = Number(row.case.split('-').at(-1));
  assert.equal(row.encoded_bytes, old.encoded_bytes + (row.codec === 'wire-native' ? n : 0));
  if (row.codec === 'candid') assert.equal(old.payload_sha256, row.payload_sha256);
  else if (!n) assert.equal(old.payload_sha256, row.payload_sha256);
  const result = {
    profile: row.profile, case: row.case, codec: row.codec, operation: row.operation,
    before_bytes: old.encoded_bytes, after_bytes: row.encoded_bytes,
    before_instructions: old.median_instructions_per_op,
    after_instructions: row.median_instructions_per_op,
    instructions_change_pct: change(old.median_instructions_per_op, row.median_instructions_per_op),
  };
  for (const mode of ['native-1', 'native-2', 'allocations', 'heap']) {
    const get = corpus => index(corpus.native[`${row.profile}-${mode}`]
      .map(r => ({profile:row.profile,...r}))).get(key(row));
    const a = get(before), b = get(after);
    assert(a && b, `missing ${mode}: ${key(row)}`);
    assert.equal(a.input_sha256, b.input_sha256);
    assert.equal(a.encoded_sha256, old.payload_sha256);
    assert.equal(b.encoded_sha256, row.payload_sha256);
    if (mode.startsWith('native')) {
      result[`before_${mode}_ns`] = a.measurement.median_ns;
      result[`after_${mode}_ns`] = b.measurement.median_ns;
      result[`${mode}_change_pct`] = change(a.measurement.median_ns,b.measurement.median_ns);
    } else for (const metric of Object.keys(a.measurement)) {
      result[`before_${mode}_${metric}`] = a.measurement[metric];
      result[`after_${mode}_${metric}`] = b.measurement[metric];
    }
  }
  rows.push(result);
}
assert.equal(rows.length, previous.size);
const write = (name, values) => {
  const columns = Object.keys(values[0]);
  fs.writeFileSync(path.join(directory,name), [columns.join(','), ...values.map(r =>
    columns.map(k => JSON.stringify(r[k] ?? '')).join(','))].join('\n')+'\n');
};
write('before-after.csv', rows);
const artifacts = corpus => corpus.identity.artifacts;
const footprints = [];
for (const a of artifacts(before)) {
  const b = artifacts(after).find(r => r.profile === a.profile && r.codec === a.codec);
  assert(b);
  // Sizes are checked against the retained modules by analyze.mjs and the
  // independent archive verifier; identities retain optimization/compiler flags.
  footprints.push({profile:a.profile, codec:a.codec, before_sha256:a.sha256, after_sha256:b.sha256});
}
write('paired-artifacts.csv',footprints);
console.log(`Matched ${rows.length} instruction rows and logical inputs across both cohorts.`);

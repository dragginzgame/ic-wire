// Compare completed codec runs; this never builds, starts a server or updates a baseline.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
const profiles = ['evaluation', 'benchmark-speed'];
const rowKey = r => [r.profile,r.case,r.codec,r.operation].join('|');
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const digest = value => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
const loadJson = file => JSON.parse(fs.readFileSync(file,'utf8'));

// Independent framing check of the reported section-10 payload. Cargo/PocketIC
// validate module semantics; gzip bytes are never accepted as Wasm metrics.
function codeBytes(bytes) {
  assert(bytes.subarray(0,8).equals(Buffer.from([0,97,115,109,1,0,0,0])), 'expected raw core Wasm');
  let at=8, code;
  while(at<bytes.length) {
    const id=bytes[at++];let length=0,shift=0,byte;
    do {
      assert(at<bytes.length && shift<35,'truncated/overflowing section length');
      byte=bytes[at++];assert(shift!==28 || byte<=15,'overflowing section length');
      length+=(byte&127)*2**shift;shift+=7;
    } while(byte&128);
    assert(at+length<=bytes.length,'truncated Wasm section');
    if(id===10) { assert(code===undefined,'duplicate code section');code=length; }
    at+=length;
  }
  return code??0;
}

function readRun(directory) {
  const identity = loadJson(path.join(directory,'instructions/identity.json'));
  const required = ['compiler','workspace_manifest_sha256','build_environment','lock_sha256',
    'server_version','server_sha256','counter','samples','sample_caller','small_repetitions','large_repetitions',
    'fixture_sha256','hotspots_fixture_sha256','bulk_fixture_sha256','packed_i16_fixture_sha256',
    'toko_fixture_sha256','derive_fixture_sha256','byte_keys_fixture_sha256','cost_smoke_fixture_sha256',
    'driver_sha256','wasm_driver_sha256'];
  for (const field of required) assert(identity[field] !== undefined, `missing identity: ${field}`);
  assert(identity.compiler.length && identity.samples === 3, 'compiler and three samples required');
  assert.equal(identity.counter,'ic0.performance_counter(0)');
  for (const field of ['native_encoder_sha256','native_decoder_sha256','native_admission_sha256'])
    assert(digest(identity[field]), `missing runtime source identity: ${field}`);
  const rows = new Map(), groups = new Map();
  for (const line of fs.readFileSync(path.join(directory,'instructions/samples.jsonl'),'utf8').trim().split('\n')) {
    const row = JSON.parse(line), key = rowKey(row);
    assert(!rows.has(key), `duplicate operation: ${key}`);
    assert(profiles.includes(row.profile) && typeof row.case === 'string' && row.case.length, `invalid case: ${key}`);
    assert(digest(row.input_sha256) && digest(row.payload_sha256), `missing frame/input identity: ${key}`);
    assert(Number.isSafeInteger(row.encoded_bytes) && row.encoded_bytes > 0, `invalid bytes: ${key}`);
    assert([1,4].includes(row.repetitions), `invalid repetitions: ${key}`);
    const samples = row.samples_instructions, callers = row.sample_caller_ids;
    assert(Array.isArray(samples) && samples.length === 3 && samples.every(n=>Number.isSafeInteger(n)&&n>=0), `invalid samples: ${key}`);
    assert(new Set(samples).size === 1, `unstable instruction samples: ${key}`);
    assert.equal(row.median_instructions_per_op,samples[0]/row.repetitions, `invalid median: ${key}`);
    assert(Array.isArray(callers) && callers.length === 3 && new Set(callers).size === 3 && callers.every(n=>Number.isSafeInteger(n)&&n>0), `invalid callers: ${key}`);
    const group = [row.profile,row.case,row.codec].join('|');
    if (groups.has(group)) assert.deepEqual(callers,groups.get(group), `inconsistent sample callers: ${key}`);
    else groups.set(group,callers);
    rows.set(key,row);
  }
  assert(rows.size, 'empty instruction evidence');
  assert.equal(new Set([...groups.values()].flat()).size,3*groups.size,'reused query callers');
  for (const [group] of groups) for (const operation of ['encode-fresh','decode-owned-or-labelled-borrowed','reject-bad-header','reject-truncated','loop-control'])
    assert(rows.has(group+'|'+operation), `missing operation: ${group}/${operation}`);
  for (const row of rows.values()) {
    assert(rows.has(rowKey({...row,profile:profiles.find(p=>p!==row.profile)})),`missing profile: ${rowKey(row)}`);
    if(row.codec.startsWith('wire-') && row.operation !== 'encode-reuse') {
      const candid = rows.get(rowKey({...row,codec:'candid'}));
      assert(candid && candid.input_sha256 === row.input_sha256,`missing/mismatched Candid input control: ${rowKey(row)}`);
    }
  }
  const [header,...lines] = fs.readFileSync(path.join(directory,'wasm.csv'),'utf8').trim().split('\n');
  assert.equal(header,'profile,codec,wasm_bytes,wasm_sha256,code_section_bytes','unsupported Wasm evidence schema');
  const artifacts = new Map();
  for (const line of lines) {
    const [profile,codec,total,sha,code] = line.split(','), key = profile+'|'+codec;
    assert(profiles.includes(profile) && ['wire','candid'].includes(codec) && !artifacts.has(key),`invalid/duplicate artifact: ${key}`);
    const bytes = fs.readFileSync(path.join(directory,`${profile}-${codec}.wasm`));
    assert.equal(hash(bytes),sha,`artifact checksum mismatch: ${key}`);
    assert.equal(bytes.length,+total,`artifact size mismatch: ${key}`);
    assert.equal(codeBytes(bytes),+code,`code-section size mismatch: ${key}`);
    assert.equal(identity.artifacts.find(a=>a.profile===profile&&a.codec===codec)?.sha256,sha,`IC artifact mismatch: ${key}`);
    const build = fs.readFileSync(path.join(directory,`${profile}-${codec}-build.jsonl`),'utf8').trim().split('\n').map(s=>JSON.parse(s));
    const guest=build.filter(r=>r.reason==='compiler-artifact'&&r.target.name==='codec-wasm');
    assert.equal(guest.length,1,`missing/duplicate guest build: ${key}`);
    assert.equal(guest[0].profile.opt_level,profile==='evaluation'?'z':'3',`compiler profile mismatch: ${key}`);
    assert(Array.isArray(guest[0].features),`missing guest features: ${key}`);
    const library=build.find(r=>r.reason==='compiler-artifact'&&r.target.name==='ic_wire');
    artifacts.set(key,{profile,codec,total:+total,code:+code,guest_fresh:guest[0].fresh,library_fresh:library?.fresh,features:[...guest[0].features].sort()});
  }
  assert.equal(artifacts.size,4,'four profile/codec artifacts required');
  return {directory,identity,required,rows,artifacts};
}

export function compareCosts(baselineDirectory,candidateDirectory) {
  const before=readRun(baselineDirectory), after=readRun(candidateDirectory);
  for (const field of before.required) assert.deepEqual(after.identity[field],before.identity[field],`identity mismatch: ${field}; make a matched baseline`);
  assert.deepEqual([...after.rows.keys()].sort(),[...before.rows.keys()].sort(),'case/operation coverage changed');
  // A relocated workspace can otherwise reuse newer cached artifacts despite
  // older copied source mtimes. Require observed compilation, even when a stale
  // host sampler could report the old runtime hashes too. Preserve dependency caches.
  for (const profile of profiles) {
    const artifact=after.artifacts.get(profile+'|wire');
    assert(artifact.library_fresh===false && artifact.guest_fresh===false,
      `${profile} reused cached library/guest; rebuild the candidate`);
  }
  for(const [key,artifact] of after.artifacts) assert.deepEqual(artifact.features,before.artifacts.get(key)?.features,`guest feature mismatch: ${key}`);
  const failures=[], instructions=[];
  for (const [key,row] of after.rows) {
    const old=before.rows.get(key);
    for(const field of ['input_sha256','payload_sha256','encoded_bytes','repetitions'])
      assert.equal(row[field],old[field],`frame/input mismatch: ${key}/${field}`);
    if (row.codec === 'candid') {
      assert.equal(row.median_instructions_per_op,old.median_instructions_per_op,`Candid control changed: ${key}`);
      continue;
    }
    if (!row.codec.startsWith('wire-') || row.operation==='loop-control') continue;
    // IC counters are deterministic. The small absolute margin avoids flagging
    // a few layout/loop instructions as a large percentage on tiny controls.
    const limit=old.median_instructions_per_op*1.05+64;
    if(row.median_instructions_per_op>limit) failures.push(`${key}: ${old.median_instructions_per_op} → ${row.median_instructions_per_op} instructions (limit ${limit})`);
    instructions.push({key,before:old.median_instructions_per_op,after:row.median_instructions_per_op});
  }
  const wasm=[];
  for(const profile of profiles) {
    assert.equal(after.identity.artifacts.find(a=>a.profile===profile&&a.codec==='candid')?.sha256,
      before.identity.artifacts.find(a=>a.profile===profile&&a.codec==='candid')?.sha256,
      `Candid control artifact changed: ${profile}`);
    const a=before.artifacts.get(profile+'|wire'), b=after.artifacts.get(profile+'|wire');
    for(const metric of ['total','code']) if(b[metric]>a[metric]) failures.push(`${profile} ${metric}: ${a[metric]} → ${b[metric]} bytes`);
    wasm.push({profile,before_total:a.total,after_total:b.total,total_delta:b.total-a.total,
      before_code:a.code,after_code:b.code,code_delta:b.code-a.code});
  }
  return {failures,instructions,wasm};
}
if (process.argv[1] && path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  try {
    assert.equal(process.argv.length,4,'usage: node check-codec-costs.mjs BASELINE_DIRECTORY CANDIDATE_DIRECTORY');
    const result=compareCosts(process.argv[2],process.argv[3]);
    for(const row of result.wasm) console.log(`${row.profile}: total ${row.before_total} → ${row.after_total} bytes (${row.total_delta>=0?'+':''}${row.total_delta}); code ${row.before_code} → ${row.after_code} bytes (${row.code_delta>=0?'+':''}${row.code_delta})`);
    console.log(`${result.instructions.length} Wire operation pairs; policy: instructions ≤ baseline × 1.05 + 64; no total/code Wasm growth.`);
    for(const failure of result.failures) console.error(failure);
    process.exitCode=result.failures.length?1:0;
  } catch(error) { console.error(error.message);process.exitCode=1; }
}

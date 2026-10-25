import fs from 'node:fs';
import {gzipSync} from 'node:zlib';
const [root, output] = process.argv.slice(2);
if (!root || !output) throw new Error('usage: analyze.mjs EXPERIMENT_DIR OUTPUT_DIR');
const variants = {};
for (const variant of ['baseline', 'candidate']) {
    const path = `${root}/standard-${variant}/instructions`;
    variants[variant] = {
        identity: JSON.parse(fs.readFileSync(`${path}/identity.json`, 'utf8')),
        samples: fs.readFileSync(`${path}/samples.jsonl`, 'utf8').trim().split('\n').map(JSON.parse),
    };
}
const key = r => [r.profile, r.case, r.codec, r.operation].join('/');
const baseline = new Map(variants.baseline.samples.map(r => [key(r), r]));
if (baseline.size !== variants.baseline.samples.length || baseline.size !== variants.candidate.samples.length) throw new Error('missing/duplicate rows');
const seen = new Set();
const changes = variants.candidate.samples.map(r => {
    const before = baseline.get(key(r));
    if (!before || seen.has(key(r))) throw new Error('unpaired row');
    seen.add(key(r));
    for (const field of ['payload_sha256', 'input_sha256', 'encoded_bytes', 'repetitions']) {
        if (r[field] !== before[field]) throw new Error(`mismatched ${field}: ${key(r)}`);
    }
    for (const sample of [r, before]) {
        if (sample.samples_instructions.length !== 3 || new Set(sample.samples_instructions).size !== 1) throw new Error('unexpected repeated counters');
    }
    return {profile:r.profile, case:r.case, codec:r.codec, operation:r.operation,
        before:before.median_instructions_per_op, after:r.median_instructions_per_op,
        percent:100*(r.median_instructions_per_op/before.median_instructions_per_op-1)};
});
fs.mkdirSync(output, {recursive:true});
fs.writeFileSync(`${output}/instructions.json.gz`, gzipSync(JSON.stringify({variants, changes})+'\n'));
const primary = changes.filter(r => r.codec === 'wire-native' && ['encode-fresh','decode-owned-or-labelled-borrowed','reject-truncated'].includes(r.operation));
fs.writeFileSync(`${output}/instruction-changes.csv`, 'profile,case,codec,operation,before,after,percent\n'+primary.map(r=>Object.values(r).join(',')).join('\n')+'\n');
console.log(JSON.stringify(primary.toSorted((a,b)=>b.percent-a.percent).slice(0,12), null, 2));

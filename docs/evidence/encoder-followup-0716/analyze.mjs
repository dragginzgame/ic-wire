import fs from 'node:fs';
import {gunzipSync} from 'node:zlib';

// Verify the frozen IC comparisons and emit every improvement/regression.
const directory = process.argv[2];
if (!directory) throw new Error('usage: node analyze.mjs EVIDENCE-DIRECTORY');
const runs = JSON.parse(gunzipSync(fs.readFileSync(`${directory}/instructions.json.gz`)));
const lines = ['variant,cohort,profile,case,codec,operation,before,after,percent'];
for (const variant of ['extend', 'one-pass', 'one-pass-shared', 'one-pass-fast']) {
    for (const cohort of ['standard', 'hundred']) {
        const baseline = new Map();
        for (const row of runs.baseline[cohort].samples) {
            const key = JSON.stringify([row.profile, row.case, row.codec, row.operation]);
            if (baseline.has(key)) throw new Error('duplicate baseline row');
            baseline.set(key, row);
        }
        const seen = new Set();
        for (const row of runs[variant][cohort].samples) {
            const key = JSON.stringify([row.profile, row.case, row.codec, row.operation]);
            const before = baseline.get(key);
            if (!before || seen.has(key)) throw new Error('unpaired row');
            seen.add(key);
            for (const field of ['payload_sha256', 'input_sha256', 'encoded_bytes', 'repetitions']) {
                if (before[field] !== row[field]) throw new Error(`mismatched ${field}`);
            }
            for (const sample of [before, row]) {
                if (sample.samples_instructions.length !== 3 || new Set(sample.samples_instructions).size !== 1) {
                    throw new Error('nonrepeatable IC counters');
                }
            }
            const b = before.median_instructions_per_op;
            const a = row.median_instructions_per_op;
            if (row.codec === 'candid' && a !== b) throw new Error('changed Candid control');
            if (row.codec.startsWith('wire')) {
                lines.push([variant, cohort, row.profile, row.case, row.codec, row.operation,
                    b, a, 100 * (a / b - 1)].join(','));
            }
        }
        if (seen.size !== baseline.size) throw new Error('missing candidate rows');
        console.error(`${variant}/${cohort}: ${seen.size} paired rows verified`);
    }
}
process.stdout.write(`${lines.join('\n')}\n`);

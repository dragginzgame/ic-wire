import fs from 'node:fs';
import {gzipSync} from 'node:zlib';
const [root, output] = process.argv.slice(2);
if (!root || !output) throw new Error('usage: native-analysis.mjs EXPERIMENT_DIR OUTPUT_DIR');
const median = values => {
    const sorted = values.toSorted((a,b)=>a-b), middle = Math.floor(sorted.length/2);
    return sorted.length % 2 ? sorted[middle] : (sorted[middle-1]+sorted[middle])/2;
};
const key = row => [row.case,row.codec,row.operation].join('/');
const runs = {};
for (const variant of ['baseline','candidate']) {
    runs[variant] = [1,2].map(run => {
        const directory = `${root}/native-${variant}-${run}`;
        const identity = JSON.parse(fs.readFileSync(`${directory}/identity.json`,'utf8'));
        const samples = fs.readFileSync(`${directory}/samples.jsonl`,'utf8').trim().split('\n').map(JSON.parse);
        return {identity, samples};
    });
}
for (const pairs of Object.values(runs)) for (const run of pairs) {
    for (const field of ['profile','mode','samples','minimum_batch_ms','case_count','compiler',
        'lock_sha256','fixture_sha256','hotspots_fixture_sha256','runner_sha256','rustflags',
        'wire_limits','candid_limits']) {
        if (JSON.stringify(run.identity[field]) !== JSON.stringify(runs.baseline[0].identity[field])) throw new Error(`native identity mismatch: ${field}`);
    }
}
const maps = Object.fromEntries(Object.entries(runs).map(([variant, pairs]) => [variant, pairs.map(run => new Map(run.samples.map(row=>[key(row),row])))]));
const baseline = runs.baseline[0].samples;
for (const pairs of Object.values(maps)) for (const map of pairs) {
    if (map.size !== baseline.length) throw new Error('missing/duplicate native rows');
}
const summary = baseline.map(row => {
    const values = {};
    for (const variant of ['baseline','candidate']) {
        const paired = maps[variant].map(map => {
            const sample = map.get(key(row));
            if (!sample) throw new Error('unpaired native row');
            for (const field of ['input_sha256','encoded_sha256','encoded_bytes']) {
                if (sample[field] !== row[field]) throw new Error(`mismatched ${field}`);
            }
            if (sample.measurement.samples_ns_per_op.length !== 7) throw new Error('sample count');
            return sample;
        });
        values[variant] = median(paired.flatMap(sample=>sample.measurement.samples_ns_per_op));
        values[`${variant}RunMedians`] = paired.map(sample=>sample.measurement.median_ns);
    }
    return {case:row.case,codec:row.codec,operation:row.operation,...values,
        percent:100*(values.candidate/values.baseline-1)};
});
fs.mkdirSync(output,{recursive:true});
fs.writeFileSync(`${output}/native.json.gz`,gzipSync(JSON.stringify({runs,summary})+'\n'));
fs.writeFileSync(`${output}/native-changes.csv`,'case,codec,operation,baseline_ns,candidate_ns,percent\n'+summary.map(row=>[row.case,row.codec,row.operation,row.baseline,row.candidate,row.percent].join(',')).join('\n')+'\n');
console.log(JSON.stringify(summary.filter(row=>row.codec==='wire-native'&&row.operation==='decode-owned-or-labelled-borrowed'&&['u16-wide-0','u16-wide-1','u16-wide-16384','i16-wide-16384','i16-small-16384','blob-1048576'].includes(row.case)),null,2));

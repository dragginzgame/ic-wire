import fs from 'node:fs';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
const [out] = process.argv.slice(2);
function artifact(path) {
    const bytes = fs.readFileSync(path);
    assert(WebAssembly.validate(bytes));
    let position = 8;
    while (position < bytes.length) {
        const id = bytes[position++];
        let length = 0, shift = 0, byte;
        do { byte = bytes[position++]; length += (byte & 127) * 2 ** shift; shift += 7; } while (byte & 128);
        if (id === 10) return { wasm_bytes: bytes.length, code_bytes: length, sha256: createHash('sha256').update(bytes).digest('hex') };
        position += length;
    }
    throw Error('missing code section');
}
const result = {};
for (const profile of ['evaluation', 'benchmark-speed']) {
    const before = artifact(`${out}/baseline-${profile}.wasm`);
    const after = artifact(`${out}/candidate-${profile}.wasm`);
    result[profile] = { before, after, delta: Object.fromEntries(['wasm_bytes', 'code_bytes'].map(key => [key, { bytes: after[key] - before[key], percent: 100 * (after[key] - before[key]) / before[key] }])) };
}
fs.writeFileSync(`${out}/artifacts.json`, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));

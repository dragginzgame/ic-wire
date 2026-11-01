import fs from 'node:fs';
import assert from 'node:assert/strict';
const base='target/deploy-0135';
const metadata=JSON.parse(fs.readFileSync(`${base}/consumer-metadata.json`));
const expected={'ic-wire':'0.13.5','ic-wire-derive':'0.13.5','ic-wire-endpoint':'0.3.0','ic-platform-types':'0.15.1','ic-platform':'0.16.0','ic-platform-macros':'0.16.0','ic-platform-runtime-guard':'0.16.0'};
const identities={};
for(const [name,version] of Object.entries(expected)){
 const packages=metadata.packages.filter(p=>p.name===name);assert.equal(packages.length,1,name);
 assert.equal(packages[0].version,version,name);assert.equal(packages[0].source,'registry+https://github.com/rust-lang/crates.io-index',name);
 identities[name]=packages[0].id;
}
for(const name of ['candid','pocket-ic','ic-testkit','ic-platform-runtime','ic-cdk','ic-cdk-executor'])assert.equal(metadata.packages.filter(p=>p.name===name).length,0,name);
const graph=fs.readFileSync(`${base}/consumer-wasm-graph.txt`,'utf8');
assert.ok(!/^(candid|pocket-ic|serde_json|ic-testkit|ic-build|ic-platform-runtime|ic-cdk-executor) v/m.test(graph));
fs.writeFileSync(`${base}/consumer-identities.json`,JSON.stringify(identities,null,2)+'\n');
console.log('Registry-only Core/derive 0.13.5, Endpoint 0.3, Platform 0.16 and canonical Types 0.15.1 pass singleton/source and Wasm isolation checks.');

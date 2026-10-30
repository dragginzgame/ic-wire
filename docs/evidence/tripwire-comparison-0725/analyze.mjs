import fs from 'node:fs';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const root=process.argv[2];
const lanes=['wire_owned','wire_borrowed','candid','tripwire_owned','tripwire_selective'];
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const stats=v=>({mean:v.reduce((a,b)=>a+b,0)/v.length,min:Math.min(...v),max:Math.max(...v),samples:v});
let server;
const all=[];
for(const profile of ['evaluation','benchmark-speed']) {
  const dir=`${root}/${profile}`;
  const calls=fs.readFileSync(`${dir}/calls.jsonl`,'utf8').trim().split('\n').map(JSON.parse);
  assert.equal(calls.length,640);
  const groups=new Map(),artifacts=[],sequences=[];
  for(const lane of lanes) {
    const wasm=fs.readFileSync(`${dir}/${lane}.wasm`),moduleHash=hash(wasm);
    const exports=WebAssembly.Module.exports(new WebAssembly.Module(wasm)).filter(e=>e.name.startsWith('canister_')).map(e=>e.name).sort();
    assert.deepEqual(exports,['canister_update consume','canister_update encode']);
    artifacts.push({lane,exports,total_bytes:wasm.length,code_bytes:Number(fs.readFileSync(`${dir}/${lane}-code-bytes.txt`,'utf8').trim()),sha256:moduleHash});
    const trace=JSON.parse(fs.readFileSync(`${dir}/${lane}-executions.json`));
    assert.equal(trace.more,false);assert.equal(trace.truncated,false);assert.equal(BigInt(trace.dropped_executions),0n);
    assert.equal(BigInt(trace.dropped_instructions),0n);assert.equal(trace.dropped_instructions_incomplete,false);
    if(server)assert.equal(trace.server_binary_hash,server);else server=trace.server_binary_hash;
    const rows=calls.filter(r=>r.lane===lane);assert.equal(rows.length,128);assert.equal(trace.executions.length,rows.length);
    rows.forEach((row,i)=>{
      const e=trace.executions[i];sequences.push(e.sequence_number);
      assert.equal(e.module_hash,moduleHash);assert.equal(e.function,`canister_update ${row.method}`);
      assert.equal(e.outcome,'reply');assert.equal(e.disposition.state,'committed');
      assert.equal(e.attempt.instructions_available,true);assert.equal(e.detail_truncated,false);assert.equal(e.dirty_pages_tracked,true);
      if(lane!=='candid') {
        const nonce=BigInt(row.nonce), body=Buffer.alloc(row.size);
        for(let j=0;j<body.length;j++)body[j]=(j*17+Number(nonce&255n))&255;
        const mode=row.mode===3?0:row.mode;
        let frame;
        if(lane.startsWith('tripwire')) {const fixed=Buffer.alloc(9);fixed[0]=mode;fixed.writeBigUInt64LE(nonce,1);frame=Buffer.concat([fixed,body]);}
        else {
          const varint=v=>{v=BigInt(v);const b=[];do{const c=Number(v&127n);v>>=7n;b.push(c|(v?128:0));}while(v);return Buffer.from(b);};
          const fixed=Buffer.alloc(12);fixed.write('IW');fixed.writeBigUInt64LE(77n,2);fixed[11]=mode;
          frame=Buffer.concat([fixed,varint(nonce),varint(row.size),body]);
        }
        assert.equal(hash(frame),row.mode===3?row.reply_sha256:row.request_sha256,'independent frame oracle');
      }
      const key=`${row.size}/${row.mode}`;
      if(!groups.has(key))groups.set(key,{size:row.size,mode:row.mode});
      const g=groups.get(key);g[lane]??=[];g[lane].push({...row,instructions:Number(e.instructions),page_instructions:Number(e.page_access_instructions),cycles:Number(row.cycles)});
      all.push({profile,call:row,execution:e});
    });
  }
  assert.equal(new Set(sequences).size,640);
  const results=[...groups.values()].map(g=>{
    for(const lane of lanes) {assert.equal(g[lane].length,8);assert.deepEqual(g[lane].map(r=>r.repetition),[0,1,2,3,4,5,6,7]);}
    for(let i=0;i<8;i++) {
      if(g.mode!==3)for(const lane of lanes)assert.equal(g[lane][i].reply_sha256,g.wire_owned[i].reply_sha256);
      for(const [a,b] of [['wire_owned','wire_borrowed'],['tripwire_owned','tripwire_selective']]){
        assert.equal(g[a][i].request_sha256,g[b][i].request_sha256);assert.equal(g[a][i].reply_sha256,g[b][i].reply_sha256);
      }
      for(const lane of lanes)assert.equal(g[lane][i].nonce,g.wire_owned[i].nonce);
    }
    const r={size:g.size,mode:['full','prefix256','reject','generate-encode-reply'][g.mode]};
    for(const lane of lanes)r[lane]={request_bytes:[...new Set(g[lane].map(r=>r.request_bytes))],reply_bytes:[...new Set(g[lane].map(r=>r.reply_bytes))],...Object.fromEntries(['instructions','page_instructions','cycles'].map(m=>[m,stats(g[lane].map(r=>r[m]))]))};
    return r;
  });
  fs.writeFileSync(`${dir}/comparison.json`,JSON.stringify({profile,server_sha256:server,physical_updates:640,excluded_warmups:5,artifacts,results},null,2)+'\n');
  console.log(profile);console.table(artifacts);console.table(results.filter(r=>r.size===1048576).map(r=>({mode:r.mode,...Object.fromEntries(lanes.map(l=>[l,r[l].instructions.mean]))})));
}
fs.writeFileSync(`${root}/samples.jsonl`,all.map(r=>JSON.stringify(r)).join('\n')+'\n');

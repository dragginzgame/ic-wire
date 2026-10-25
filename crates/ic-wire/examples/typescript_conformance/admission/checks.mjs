{
  const { readFileSync, writeFileSync } = await import('node:fs');
  const { createCodec } = await import('./compiled/wire-runtime.js');
  const cases = JSON.parse(readFileSync(new URL('./admission.json', import.meta.url)));
  const succeeds = action => { try { action(); return true; } catch (error) { if(error.code === 'ERR_ASSERTION') throw error; assert.ok(['limit','malformed','value','identity'].includes(error.code)); return false; } };
  // Frozen minima: Rust encode/decode, then TS encode/decode; nodes then depth.
  // These differences are deliberate qualification controls, not parity claims.
  const expected = {
    scalar:[[4,1,1,1],[3,1,1,1]], record:[[16,6,4,4],[5,3,3,3]],
    option:[[8,2,2,2],[4,2,2,2]], newtype:[[8,2,2,2],[4,2,2,2]],
    transparent:[[4,1,1,1],[3,1,1,1]], 'enum-empty':[[4,3,2,2],[3,2,2,2]],
    'recursive-empty':[[8,5,2,2],[4,2,2,2]], 'recursive-list':[[12,9,4,4],[5,4,4,4]],
    'recursive-option':[[12,8,4,4],[5,4,4,4]], mutual:[[24,10,6,6],[8,6,6,6]],
    'recursive-record':[[16,6,4,4],[6,4,4,4]],
    bytes:[[4,1,1,1],[3,1,1,1]], packed:[[4,1,1,1],[3,1,1,1]]
  };
  const report = [];
  for (const c of cases) {
    const value = c.name === 'bytes' ? new Uint8Array(c.value) : c.name === 'packed' ? new Int16Array(c.value) : c.value;
    const minima = {};
    for (const row of c.rows) {
      const codec = createCodec({...row.budgets,id:'17',request:c.root,response:c.root},c.types);
      const encode = row.direction ? codec.encodeResponse : codec.encodeRequest;
      const decode = row.direction ? codec.decodeResponse : codec.decodeRequest;
      const encoded = succeeds(() => assert.deepEqual([...encode(value)], c.frames[row.direction]));
      const decoded = succeeds(() => assert.equal(JSON.stringify(decode(new Uint8Array(c.frames[row.direction]))), JSON.stringify(value)));
      if (row.axis === 'bytes') { assert.equal(encoded,row.rustEncode); assert.equal(decoded,row.rustDecode); }
      for (const [index, [runtime, ok]] of [['rustEncode',row.rustEncode],['rustDecode',row.rustDecode],['tsEncode',encoded],['tsDecode',decoded]].entries()) {
        const minimum = row.axis === 'bytes' ? c.frames[row.direction].length : expected[c.name][row.axis === 'nodes' ? 0 : 1][index];
        assert.equal(ok, row.limit >= minimum, `${c.name}/${row.direction}/${row.axis}/${runtime}/${row.limit}`);
        const key = `${row.direction}/${row.axis}/${runtime}`;
        if (ok && minima[key] === undefined) minima[key] = row.limit;
      }
    }
    const codec = createCodec({id:'17',request:c.root,response:c.root,requestBytes:4096,responseBytes:4096,requestNodes:4096,responseNodes:4096,requestDepth:128,responseDepth:128},c.types);
    for (let direction=0;direction<2;direction++) {
      const decode = direction ? codec.decodeResponse : codec.decodeRequest;
      for(const frame of c.malformed[direction]) assert.equal(succeeds(()=>decode(new Uint8Array(frame))),false);
      for(const mutation of c.mutations[direction]) assert.equal(succeeds(()=>decode(new Uint8Array(mutation.bytes))),mutation.accepted);
    }
    report.push({name:c.name,minima});
  }
  writeFileSync(new URL('./admission-results.json',import.meta.url), JSON.stringify(report,null,2));
  console.log(`Cross-runtime admission: ${cases.length} shapes checked in both directions`);
}

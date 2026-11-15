// Native investigation driver. Does not publish, alter product profiles or tune codecs.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdirSync, mkdtempSync, readFileSync, readdirSync, writeFileSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {cpus, platform, release, arch} from 'node:os';

const [mode = 'measure', ...selection] = process.argv.slice(2);
assert(['measure','allocations','profile'].includes(mode), 'select measure, allocations or profile');
assert(!selection.includes('--mode'), 'the driver selects --mode; pass case/operation selectors instead');
const profile = process.env.CODEC_PROFILE || 'codec-profile';
assert(['codec-profile','codec-profile-speed'].includes(profile), 'unsupported native profile');
for (const [key,value] of Object.entries(process.env)) {
  if (['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_BUILD_TARGET'].includes(key) || /^CARGO_(PROFILE_|TARGET_.*_RUSTFLAGS)/.test(key)) assert(!value, `unset ${key} for the fixed native comparison`);
}
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
mkdirSync('qualification/target', {recursive:true});
const output = mkdtempSync(resolve(`qualification/target/codec-${mode}-`));
console.log(`Native codec evidence: ${output}`);
const report = {mode, profile, selection, node:process.version, host:{platform:platform(),release:release(),arch:arch(),cpu:cpus()[0]?.model}, completed:false};
function command(program, args, file) {
  const result = spawnSync(program, args, {encoding:'utf8', maxBuffer:64*1024*1024, stdio:['ignore','pipe','pipe']});
  if (file) { writeFileSync(join(output,file),result.stdout || ''); writeFileSync(join(output,`${file}.stderr`), result.stderr || ''); }
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${program} failed (${result.status}): ${result.stderr}`);
  return result.stdout.trim();
}
try {
  report.compiler = command('rustc',['-Vv']);
  report.base_commit = command('git',['rev-parse','HEAD']);
  const inputs = ['Cargo.toml','rust-toolchain.toml','qualification/Cargo.toml','qualification/Cargo.lock','qualification/compatibility/Cargo.toml','qualification/codec/run.mjs'];
  function sources(directory) {
    for (const entry of readdirSync(directory,{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name))) {
      const path = join(directory,entry.name);
      if(entry.isDirectory()) sources(path); else inputs.push(path);
    }
  }
  sources('crates/ic-wire/src'); sources('qualification/compatibility/benches/codec');
  report.source_inputs = Object.fromEntries(inputs.sort().map(path=>[path,sha(readFileSync(path))]));
  report.source_sha256 = sha(JSON.stringify(report.source_inputs));
  const buildArgs = ['bench','--manifest-path','qualification/Cargo.toml','-p','ic-wire-compatibility','--bench','codec','--no-run','--profile',profile,'--locked','--offline','--message-format=json'];
  if(mode === 'allocations') buildArgs.push('--features','bench-allocations');
  report.build = ['cargo',...buildArgs];
  const messages = command('cargo',buildArgs,'build.jsonl').split('\n').filter(Boolean).map(JSON.parse);
  const binary = messages.findLast(message=>message.reason==='compiler-artifact' && message.target.name==='codec' && message.executable)?.executable;
  assert(binary,'Cargo did not report the codec executable');
  report.binary_sha256 = sha(readFileSync(binary));
  report.catalog = JSON.parse(command(binary,['--list']));
  if(selection.includes('--list')) {
    console.log(JSON.stringify(report.catalog,null,2));
    report.completed = true;
    writeFileSync(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');
    process.exit(0);
  }
  const args = [...selection,'--mode', mode === 'profile' ? 'profile' : 'measure'];
  report.run = [binary,...args];
  if (mode === 'profile') {
    report.perf_version = command('perf',['--version']);
    const perfArgs = ['record','--event','cpu-clock:u','--freq','499','--call-graph','dwarf','--output',join(output,'perf.data'),'--',binary,...args];
    report.profiler = ['perf',...perfArgs];
    command('perf',perfArgs,'profile.jsonl');
    command('perf',['report','--stdio','--no-children','--input',join(output,'perf.data')],'hotspots.txt');
    console.log(`Function samples: ${join(output,'hotspots.txt')}`);
  } else {
    const rows = command(binary,args,'rows.jsonl').split('\n').filter(Boolean).map(JSON.parse);
    assert(rows.length > 0 && rows.every(row=>row.measurements), 'no measurement rows');
    report.rows = rows;
    console.log(`${rows.length} ${mode} rows; ${join(output,'report.json')}`);
    if(mode === 'measure') {
      const comparisons = rows.filter(row=>row.supplier==='candidate' && row.operation!=='loop').flatMap(row=>{
        const old = rows.find(other=>other.supplier==='reference' && other.case===row.case && other.size===row.size && other.representation===row.representation && other.operation===row.operation);
        if(!old) return [];
        return [{case:row.case,size:row.size,representation:row.representation,operation:row.operation,candidate_ns:row.measurements.median_ns,reference_ns:old.measurements.median_ns,ratio:row.measurements.median_ns/old.measurements.median_ns}];
      });
      report.comparisons = comparisons;
      for(const row of comparisons.sort((a,b)=>(b.candidate_ns-b.reference_ns)-(a.candidate_ns-a.reference_ns)).slice(0,10)) console.log(`${row.case}/${row.representation}/${row.size} ${row.operation}: ${row.candidate_ns.toFixed(1)} vs ${row.reference_ns.toFixed(1)} ns (${row.ratio.toFixed(2)}x)`);
    }
  }
  report.completed = true;
} catch(error) {
  report.failure = String(error);
  console.error(report.failure);
  process.exitCode = 1;
} finally {
  writeFileSync(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');
}

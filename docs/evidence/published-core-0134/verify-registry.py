import hashlib, io, json, sys, tarfile, tomllib
from pathlib import Path

name, version, commit = sys.argv[1:]
root = Path(__file__).resolve().parent
records = [json.loads(s) for s in (root / f'{name}-index-after.jsonl').read_text().splitlines()]
record = next(r for r in records if r['vers'] == version)
assert record['name'] == name and not record['yanked']
archive = root / 'upload-archives' / f'{name}-{version}.crate'
data = archive.read_bytes()
digest = hashlib.sha256(data).hexdigest()
assert digest == record['cksum']
candidates = [Path('target/package') / sub / archive.name for sub in ['', 'tmp-crate', 'tmp-registry']]
assert any(p.is_file() and p.read_bytes() == data for p in candidates), 'registry archive differs from prepared/uploaded archive'
with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as tar:
    matches = [m for m in tar.getmembers() if m.name.endswith('/.cargo_vcs_info.json')]
    assert len(matches) == 1 and matches[0].isfile()
    vcs = json.load(tar.extractfile(matches[0]))
    assert vcs['git']['sha1'] == commit and not vcs['git'].get('dirty', False)
    assert vcs['path_in_vcs'] == f'crates/{name}'
    prefix = f'{name}-{version}/'
    assert all(not m.name.startswith((prefix + 'tests/', prefix + 'drivers/', prefix + 'docs/evidence/')) for m in tar.getmembers())
    manifest = tomllib.loads(tar.extractfile(prefix + 'Cargo.toml').read().decode())
    assert manifest['package']['name'] == name and manifest['package']['version'] == version
    assert not any(n in manifest.get('dependencies', {}) for n in ['candid', 'ic-testkit', 'ic-build'])
    if name == 'ic-wire':
        assert manifest['dependencies']['ic-wire-derive']['version'] == version
        assert b'pub use serde_bytes::ByteArray;' in tar.extractfile(prefix + 'src/lib.rs').read()
        source = tar.extractfile(prefix + 'src/native/mod.rs').read()
        assert b'if nodes < count || depth > self.depth || heap < size' in source
(root / f'{name}-{version}-registry.json').write_text(json.dumps(record, indent=2) + '\n')
(root / f'{name}-{version}-vcs.json').write_text(json.dumps(vcs, indent=2) + '\n')
(root / f'{name}-{version}-verified.json').write_text(json.dumps({
    'package': name, 'version': version, 'commit': commit, 'sha256': digest,
    'archive_matches_upload': True, 'yanked': False, 'clean_source_verified': True,
}, indent=2) + '\n')
print(f'{name} {version}: registry checksum, upload archive, clean release source and package scope verified')

#!/usr/bin/env python3
"""Pack mods/<id> as completed/<id>-<version>.hnmod (source only), the way the
template's tools/package.py does. Usage: tools/package.py <id>..."""
import hashlib, json, sys, tomllib, zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def package(mod_id):
    root = ROOT / 'mods' / mod_id
    compat = json.loads((root / 'compatibility.json').read_text())
    bundle = root / 'mixins' / mod_id
    mixin = tomllib.loads((bundle / 'horizon-mixin.toml').read_text())
    mod = mixin['mod']
    assert mod['id'] == mod_id, 'folder and mod id differ'
    assert mod_id.replace('_', '').isalnum() and mod_id.islower()
    assert mixin['compatibility']['horizon_revision'] == compat['horizon_revision']
    manifest = dict(schema=1, id=mod_id, version=mod['version'], game='26.3',
                    horizon=compat['horizon_revision'], mixins=compat['horizon_mixins_revision'])
    if 'hooks' in compat:
        manifest['hooks'] = compat['hooks']
    files = {'horizon-native.json': (json.dumps(manifest, indent=2, sort_keys=True) + '\n').encode()}
    for path in sorted(bundle.iterdir()):
        if path.is_file() and path.suffix in ('.rs', '.toml'):
            files[path.relative_to(root).as_posix()] = path.read_bytes()
    out = ROOT / 'attempted' / f"{mod_id}-{mod['version']}.hnmod"
    with zipfile.ZipFile(out, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    print(out.name, out.stat().st_size, hashlib.sha256(out.read_bytes()).hexdigest()[:16])

for mod_id in sys.argv[1:]:
    package(mod_id)

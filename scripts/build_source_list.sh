#!/bin/sh
# 프로젝트 루트의 Release 패키지를 확인하고 정적 소스 목록을 생성한다.
set -eu
repository_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repository_dir"

# Aidoku CLI는 파일 읽기 실패에도 종료 코드 0을 반환할 수 있다.
python3 - <<'PY'
import json
import zipfile
from pathlib import Path

package = Path('package.aix')
if not package.is_file() or not package.stat().st_size:
    raise SystemExit('package.aix가 없거나 비어 있습니다.')
with zipfile.ZipFile(package) as archive:
    if archive.testzip() is not None:
        raise SystemExit('패키지 ZIP이 손상되었습니다.')
    for name in ('main.wasm', 'source.json', 'settings.json', 'icon.png'):
        if 'Payload/' + name not in archive.namelist():
            raise SystemExit('패키지 필수 파일이 누락되었습니다: ' + name)
    if archive.read('Payload/main.wasm')[:8] != b'\0asm\x01\0\0\0':
        raise SystemExit('Wasm 파일 헤더가 올바르지 않습니다.')
    json.loads(archive.read('Payload/source.json'))
    json.loads(archive.read('Payload/settings.json'))
print('package.aix 구조: 확인')
PY

aidoku verify package.aix
aidoku build -o public -n 'BookOrbit Sources' package.aix

python3 - <<'PY'
import json
import zipfile
from pathlib import Path

root = Path('public').resolve()
index = json.loads((root / 'index.min.json').read_text())
with zipfile.ZipFile('package.aix') as archive:
    info = json.loads(archive.read('Payload/source.json'))['info']
sources = index['sources']
if len(sources) != 1:
    raise SystemExit('소스 목록에는 BookOrbit 하나만 있어야 합니다.')
source = sources[0]
if source['id'] != info['id'] or source['version'] != info['version']:
    raise SystemExit('소스 목록의 ID 또는 버전이 패키지와 다릅니다.')
for key in ('downloadURL', 'iconURL'):
    asset = (root / source[key]).resolve()
    if not asset.is_relative_to(root) or not asset.is_file() or not asset.stat().st_size:
        raise SystemExit('소스 목록의 배포 파일이 없거나 경로가 잘못되었습니다: ' + key)
if (root / source['downloadURL']).read_bytes() != Path('package.aix').read_bytes():
    raise SystemExit('배포할 AIX가 Release 패키지와 다릅니다.')
print(f"소스 목록: {source['id']} 버전 {source['version']}, AIX 일치 확인")
PY

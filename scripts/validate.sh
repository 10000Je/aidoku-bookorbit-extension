#!/bin/sh
# 네트워크 접근 가능한 개발 환경에서 전체 검증 후 실제 패키지만 검사한다.
set -eu
repository_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repository_dir"

cargo fmt --check
cargo check --locked
cargo test --locked
python3 -m unittest discover -s tests -p 'test_*.py'
cargo build --locked --release --target wasm32-unknown-unknown
aidoku package --help
aidoku verify --help
aidoku package
# CLI가 읽기 실패 시 종료 코드 0을 반환할 수 있으므로 존재·ZIP·Wasm을 먼저 확인한다.
python3 - <<'PY'
import json
import zipfile
from pathlib import Path

package = Path('package.aix')
if not package.is_file() or not package.stat().st_size:
    raise SystemExit('package.aix가 생성되지 않았습니다.')
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

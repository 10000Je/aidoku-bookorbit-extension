# 개발 및 배포

모든 명령은 프로젝트 루트에서 실행합니다. 설치·사용법은 [README](../README.md), 프로젝트 관계와 외부 라이선스는 [오픈소스 고지](../THIRD_PARTY_NOTICES.md)를 참고하세요.

## 프로젝트 구조

| 경로 | 역할 |
|---|---|
| `src/config.rs`, `auth.rs` | 서버 URL 검증, 프록시 경로 유지, 비밀 값 및 인증 헤더 |
| `src/api.rs`, `models.rs`, `error.rs` | REST 요청·재시도·응답 파싱·타입·오류 분류 |
| `src/reader.rs` | 파일 역할 필터, 챕터 정렬, 페이지 URL 생성 |
| `src/source.rs` | Aidoku SDK 어댑터. Wasm 대상에서만 컴파일 |
| `res/` | 소스 메타데이터, 그룹 설정, 128×128 불투명 아이콘 |
| `tests/` | 합성 JSON fixture, Rust 테스트, probe의 Python 테스트 |
| `scripts/` | 전체 검증과 명시적 실제 서버 점검 |
| `docs/` | [초기 감사](INITIAL_AUDIT.md), [API 호환성](API_COMPATIBILITY.md), [실행 결과](VALIDATION.md) |

## 개발 환경과 빌드

Rust 2024를 지원하는 최신 도구 체인을 사용하세요. 이번 환경의 컴파일러는 Rust 1.100.0입니다. SDK와 CLI는 같은 공식 revision으로 맞춥니다. 최초 의존성 설치에는 GitHub·crates.io에 접근할 수 있어야 합니다.

```sh
rustup target add wasm32-unknown-unknown
cargo install --git https://github.com/Aidoku/aidoku-rs \
  --rev ae6485e3c14ecd1fd5497d170eae884c68d04a87 --locked aidoku-cli

aidoku --help
aidoku package --help
aidoku verify --help
sh scripts/validate.sh
```

전체 검증 스크립트는 다음 순서로 실행합니다. Python은 3.9 이상이며 표준 라이브러리만 필요합니다.

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
python3 -m unittest discover -s tests -p 'test_*.py'
cargo build --locked --release --target wasm32-unknown-unknown
aidoku package
aidoku verify package.aix
```

생성된 Wasm은 `target/wasm32-unknown-unknown/release/bookorbit_aidoku_source.wasm`, 패키지는 **프로젝트 루트의 `package.aix`**입니다. 내부 Wasm이 release 빌드와 일치하는 것을 확인했습니다. 현재 산출물 크기와 SHA-256은 [검증 기록](VALIDATION.md)에 적었습니다. 기본 호스트 `cargo check/test`와 별도로 전체 Wasm release 빌드도 통과했습니다.

CLI가 파일 읽기 실패에도 종료 코드 0을 반환하는 경우가 있어 스크립트는 실제 파일·ZIP 구성·Wasm 헤더를 먼저 검사합니다. `aidoku verify`는 설치 및 실제 네트워크 동작을 대신 검증하지 않습니다.

## 로컬 패키지를 Aidoku에 설치하기

이 체크아웃의 `package.aix`는 빌드·CLI 검증을 통과했습니다. 다음 절차로 기기에 설치하여 실제 서버 동작을 확인하세요.

1. 컴퓨터와 아이폰/아이패드를 같은 네트워크에 연결합니다. 컴퓨터의 8080 포트 접근을 허용합니다.
2. `aidoku serve package.aix`를 실행합니다. 서버를 실행한 터미널을 유지합니다.
3. CLI가 표시하는 `http://컴퓨터의-LAN-IP:8080/index.min.json`을 복사합니다. 아이폰에서 컴퓨터의 `localhost` 주소를 사용하지 마세요.
4. Aidoku의 소스 목록 관리에서 해당 URL을 추가하고 목록을 새로 고칩니다. 메뉴 이름은 앱 버전에 따라 다를 수 있습니다.
5. 표시된 **BookOrbit** 소스를 설치합니다.
6. 소스 설정에서 서버 기본 URL과 읽기 전용 API 키를 입력하고 목록을 다시 엽니다.

AirDrop 등으로 `.aix`를 기기에 전달해 Aidoku로 여는 방법도 SDK 문서에 안내되어 있습니다. 기기에서 파일 연결 메뉴가 제공되지 않으면 소스 목록 방식을 사용하세요.

개인 소스 목록도 `public/index.min.json`에 생성합니다. 버전 5 패키지는 `public/sources/ko.bookorbitprivate-v5.aix`, 아이콘은 `public/icons/ko.bookorbitprivate-v5.png`에 있습니다. 새 패키지로 목록을 갱신하려면 `sh scripts/build_source_list.sh`를 실행하세요. 패키지와 목록의 ID·버전·AIX 일치 여부를 검사합니다. `public/`은 생성물로 Git 추적에서 제외합니다. 패키지와 소스 목록에는 개인 URL/API 키가 들어가지 않습니다.

## GitHub Pages 소스 목록

Aidoku의 소스 목록 관리에서 다음 URL을 추가하고 새로 고침한 뒤 **BookOrbit**을 설치하세요. 설치 후 소스 설정에 개인 서버 URL과 읽기 전용 API 키를 입력합니다.

```text
https://10000je.github.io/aidoku-bookorbit-extension/index.min.json
```

GitHub Actions의 [`Deploy Aidoku source list`](../.github/workflows/pages.yml)는 공개한 Release의 `package.aix`를 내려받아 고정된 공식 Aidoku CLI로 목록을 생성하고 GitHub Pages에 배포합니다. 배포되는 AIX는 Release 첨부 파일과 동일합니다. GitHub 저장소의 **Settings → Pages → Source**는 **GitHub Actions**로 설정합니다.

새 버전은 `res/source.json`의 정수 `version`을 올리고 전체 검증·패키징한 뒤, 변경을 커밋·푸시하고 새 정식 Release에 `package.aix`를 첨부해 게시하세요. 이 이벤트가 목록 배포를 자동 실행합니다. 초안과 사전 Release는 자동 배포하지 않습니다. 현재 Pages 환경에서는 Release 태그에서의 배포가 보호 규칙으로 거부되므로, 게시 후 허용된 `main` 브랜치에서 아래 수동 배포를 실행하세요. 환경 보호 규칙은 변경하지 않습니다.

```sh
gh workflow run pages.yml --ref main
# 특정 Release를 배포하려면 태그를 지정합니다.
gh workflow run pages.yml --ref main -f release_tag=v5
```

배포 결과는 저장소의 **Actions**에서 확인할 수 있습니다. Pages에 올리는 내용은 소스 목록·확장 패키지·아이콘이며, BookOrbit 서버 자체나 서버의 도서는 게시하지 않습니다.

## 실제 서버 점검

자동 테스트는 실제 서버에 연결하지 않습니다. 다음 도구는 **사용자가 실행할 때만** GET과 읽기 전용 POST 검색을 수행합니다. 데이터 수정·업로드·진도 쓰기를 하지 않습니다. 키는 채팅에 전달하지 말고 로컬에서 입력하세요.

```sh
export BOOKORBIT_URL='https://books.example.com/shelf'
export BOOKORBIT_API_KEY="$(python3 -c 'import getpass; print(getpass.getpass("읽기 전용 API 키: "))')"
python3 scripts/probe_bookorbit.py --query '검색할 제목'
# 특정 도서의 모든 페이지를 순차 점검하려는 경우:
python3 scripts/probe_bookorbit.py --book-id 42 --all-pages
unset BOOKORBIT_API_KEY
```

기본 점검은 서버 버전·API 키 인증·검색·다음 목록 페이지·상세·표지·각 만화 파일의 첫/마지막 이미지 응답을 확인합니다. 전체 이미지 파일을 메모리에 쌓지 않습니다. 모든 HTTP 리다이렉트를 거부합니다. 빈 라이브러리이면 챕터·이미지를 검사하지 않았다고 표시합니다. 자격 증명 미설정 시 `SKIP`과 종료 코드 2, 실패 시 1을 반환합니다. 이 도구의 성공도 Aidoku Wasm 실행이나 iOS 디코딩 성공을 의미하지 않습니다.

## Git 관리

기본 브랜치는 `main`이며 GitHub 원격 저장소는 `https://github.com/10000Je/aidoku-bookorbit-extension.git`입니다. 소스·설정 스키마·아이콘·합성 fixture·문서·`Cargo.lock`을 추적합니다. `target/`, `package.aix`, `public/`, Python 캐시와 `.env` 파일은 Git 추적에서 제외합니다.

기존 버전 3은 `ffec300`, CB7 변경은 `22e00fe` 커밋에 기록되어 있습니다. [초기 감사](INITIAL_AUDIT.md), [API 호환성](API_COMPATIBILITY.md), [검증 기록](VALIDATION.md)에서 설계 근거와 실행 결과를 확인할 수 있습니다.

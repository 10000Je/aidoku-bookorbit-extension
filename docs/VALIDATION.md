# 검증 결과

실행일: 2026-11-17. 현재 산출물은 **소스 버전 4**이며 CB7을 추가했다. Rust 45개·Python 9개 테스트, 전체 SDK/Wasm release 빌드, 패키징과 CLI 검증을 통과했다. **CB7 실제 파일의 서버 추출·아이폰 열람은 미실행**이다. 라이브러리·시리즈 기능의 실기기 검증도 미실행이며, 이전 버전 2의 기본 사용 검증 8개는 사용자 보고로 확인한 별도 결과다.

## GitHub Pages 배포 검증

실행일: **2026-11-18 KST**. 공개 저장소 `10000Je/aidoku-bookorbit-extension`의 `v4` Release에 첨부한 `package.aix`로 소스 목록을 생성했다. Pages의 게시 방식은 GitHub Actions이며 HTTPS 강제가 활성화되어 있다. 배포 설정은 `372589d` 커밋에 기록했다.

Aidoku에 등록할 주소: [BookOrbit 소스 목록](https://10000je.github.io/aidoku-bookorbit-extension/index.min.json).

| 검사 | 결과 | 범위 |
|---|---|---|
| 워크플로 YAML·셸 문법 | PASS | YAML 파싱, 워크플로 셸 단계의 `bash -n`, 배포 스크립트의 `sh -n` |
| `sh scripts/build_source_list.sh` | PASS | 로컬 AIX의 ZIP·Wasm·JSON 구조, 공식 CLI 검증, 목록 생성과 ID·버전·AIX 일치 검사 |
| GitHub Actions 첫 수동 배포 | PASS | [실행 44110843403](https://github.com/10000Je/aidoku-bookorbit-extension/actions/runs/44110843403)의 build·deploy 모두 success |
| 공개 목록·AIX·아이콘 | PASS | 각 URL에서 HTTP 200, 각각 331·198,062·3,399바이트 |
| 공개 파일 일관성 | PASS | 목록의 ID·version=4·상대 경로 확인, Pages AIX = v4 Release AIX = 로컬 AIX, 아이콘 = 패키지 내부 아이콘 |
| 아이폰에서 Pages 목록 등록·업데이트 | 미실행 | 공개 HTTP 응답과 파일 검증까지 수행했으며 Aidoku 앱 동작은 별도 확인 필요 |

공개 AIX의 SHA-256은 아래 버전 4 패키지와 같은 `6e705d009fa58a58fa26d0532c998bc8ee49b086df0c317362acb30c3a6e88d4`다. 이번 배포 작업은 Rust 코드나 AIX를 변경하지 않았으며, 기존 Rust/Python 테스트와 Wasm 컴파일을 다시 실행하지 않았다.

새 정식 Release의 게시 이벤트가 소스 목록 배포를 자동 실행한다. 초안·사전 Release는 자동 배포하지 않는다. 기존 `v4` Release는 워크플로 추가 이전에 게시되었으므로 `workflow_dispatch`와 `release_tag=v4`를 사용했다. 수동 재실행 및 새 버전 배포 절차는 README에 기록했다.

## 버전 4 직접 실행 결과

SDK `ae6485e`와 BookOrbit `v3.8.0 (450c18e)` 계약을 유지했다. 공식 서버의 `CbzService`가 CB7을 `getSevenZPages`로 처리하고, 기존 `/cbz/files/{fileId}/pages` 및 페이지 이미지 경로를 제공하는 것을 확인했다. 확장은 검색 형식과 primary/content 파일 판정에 CB7을 추가하고 기존 인증·소속 확인·페이지 순서 경로를 사용한다.

| 검사 | 결과 | 범위 |
|---|---|---|
| `sh scripts/validate.sh` | PASS | 형식 → 호스트 검사 → Rust/Python 테스트 → Wasm → 패키징 → CLI, 종료 코드 0 |
| `cargo fmt --check`, `cargo check --locked` | PASS | Rust 형식과 핵심 모듈 |
| `cargo test --locked` | PASS | Rust 45개: 클라이언트 43개, 리소스 2개 |
| `python3 -m unittest discover -s tests -p 'test_*.py'` | PASS | probe 9개, 실제 서버 연결 없음 |
| `cargo build --locked --release --target wasm32-unknown-unknown` | PASS | 고정 SDK와 전체 어댑터 |
| `aidoku package`, `aidoku verify package.aix` | PASS | 실제 ZIP·Wasm 및 CLI의 모든 검사 yes |
| `aidoku build -o public -n 'BookOrbit Private Sources' package.aix` | PASS | 버전 4 목록·패키지 사본 생성 |
| 패키지 일관성 | PASS | 소스 ID 유지·version=4·ZIP 내 Wasm = release·루트 패키지 = 배포 사본 |
| 실제 CB7 서버 추출·아이폰 열람 | 미실행 | 합성 JSON 및 경로 검증으로 실제 압축 해제/디코딩 성공을 주장하지 않음 |

추가한 Rust 테스트 4개는 CB7 전용 도서의 전체 검색/라이브러리 표시, primary/content와 대소문자 지원, sidecar·일반 `7z` 제외, 기존 키 유지, 시리즈 챕터 변환, 동일한 `/cbz` 페이지 경로와 첫/마지막 페이지 순서, 파일 소속·422/404·0페이지 처리를 확인한다. Python 테스트 1개는 probe가 CB7만 있는 도서에서 검색 필터와 각 파일의 첫/마지막 이미지를 요청하는지 확인한다. `cb7-book-detail.json`은 실제 아카이브가 아닌 합성 fixture다.

현재 `package.aix`는 **198,062바이트**, release Wasm은 **536,599바이트**다. 배포 사본은 `public/sources/ko.bookorbitprivate-v4.aix`, 아이콘은 `public/icons/ko.bookorbitprivate-v4.png`이며 `public/index.min.json`이 이를 참조한다. 초기 로컬 검증 시점에는 외부 호스팅에 게시하지 않았으며, 이후 Pages 배포 결과는 위 절에 기록했다.

패키지 SHA-256:

```text
6e705d009fa58a58fa26d0532c998bc8ee49b086df0c317362acb30c3a6e88d4
```

로컬 Git 저장소를 `main` 브랜치로 초기화하고 기존 버전 3을 `ffec300` 커밋에 보존했다. CB7 변경은 `22e00fe` 커밋에 기록했다. 소스·문서·아이콘·테스트·잠금 파일을 추적하고 빌드/배포 산출물과 `.env`는 제외한다. 초기화 시점에는 원격 저장소가 없었으며, 이후 GitHub 저장소를 `origin`에 연결했다.

## 이전 버전 3 검증 이력

아래는 라이브러리·시리즈 기능 추가 당시의 **버전 3** 결과이며 현재 버전 4 산출물의 크기·해시와 구분한다.

## 버전 3 직접 실행 결과

SDK와 서버 계약 기준은 이전과 같은 `aidoku-rs ae6485e` 및 BookOrbit `v3.8.0 (450c18e)`다. 현재 사용자 서버·앱의 정확한 버전과 API 자격 증명은 제공받지 않았다.

| 검사 | 결과 | 범위 |
|---|---|---|
| `sh scripts/validate.sh` | PASS | 아래 형식·검사·테스트·빌드·패키징·CLI 검증을 연속 실행, 종료 코드 0 |
| `cargo fmt --check` | PASS | Rust 형식 |
| `cargo check --locked` | PASS | 핵심 호스트 모듈 |
| `cargo test --locked` | PASS | Rust 41개: 클라이언트 39개, 리소스 2개 |
| `python3 -m unittest discover -s tests -p 'test_*.py'` | PASS | 기존 probe 테스트 8개 |
| `cargo build --locked --release --target wasm32-unknown-unknown` | PASS | 동적 목록·시리즈 어댑터를 포함한 실제 SDK 빌드 |
| `aidoku package` | PASS | 버전 3 `package.aix`, ZIP·필수 구성·Wasm 헤더 확인 |
| `aidoku verify package.aix` | PASS | 최소 API 버전·필수 export·아이콘·JSON 스키마 전부 yes |
| `aidoku build -o public -n 'BookOrbit Private Sources' package.aix` | PASS | 목록의 version=3 및 v3 패키지·아이콘 경로 확인 |
| 패키지 일관성 | PASS | ZIP 내 Wasm = release 산출물, `get_listings` 실제 export, 루트 패키지 = public 배포 사본 |
| 실제 서버·아이폰의 라이브러리/시리즈 | 미실행 | 합성 fixture와 빌드만으로 실기기 성공을 주장하지 않음 |
| EPUB | 조사 완료, 열람 미구현 | SDK 텍스트 형식·앱 공식 릴리스 노트·EPUB PR 확인. 실제 EPUB 열람 검증 없음 |

추가한 클라이언트 테스트 10개는 라이브러리 이름/순서·빈 응답·중복, ID 기반 목록 경로, 대표 도서 변경 및 라이브러리 간 시리즈 키 구분, 기존 숫자 키 호환, 서버 권수 순서 유지, 41권 전 페이지 조회, 중복/전체 수/소속 변경 및 누락 페이지 거부, 20,000권 한도·부분 실패, EPUB/sidecar/missing 제외, 상세의 membership 및 도서-파일 소속 검사를 확인한다. fixture는 공식 서버 코드에 근거한 합성 응답이다.

버전 3 당시 산출물:

| 경로 | 크기 |
|---|---|
| `package.aix` | 197,986바이트 |
| `target/wasm32-unknown-unknown/release/bookorbit_aidoku_source.wasm` | 536,347바이트 |
| `public/sources/ko.bookorbitprivate-v3.aix` | 루트 패키지와 바이트 일치 |
| `public/icons/ko.bookorbitprivate-v3.png` | 목록에서 참조하는 아이콘 |
| `public/index.json`, `public/index.min.json` | 버전 3 개인 소스 목록 |

패키지 SHA-256:

```text
0d1c8afafbe05bb623f5b2642672f13a8c7bc4865b0e4a029e3527a624fb0dd6
```

버전 3에서 기기로 확인할 항목은 서버 라이브러리 이름/순서와 선택한 목록의 도서 일치, `드래곤` 한 작품 안의 c01/c02 챕터와 연속 열람, 40권 초과 시리즈, 기존 개별 도서와 읽던 위치 유지다. 기존 개별 도서 기록을 새 시리즈로 자동 병합하지 않는다. 소스 목록은 로컬 파일만 갱신했으며 외부 호스팅에 게시하지 않았다. 기본 probe는 기존 개별 도서/이미지 REST 점검용이며 새 라이브러리·시리즈 전용 서버 점검을 대신하지 않는다.

## 이전 버전 2 검증 이력

**이하 환경·실행 기록·산출물·실기기 보고는 기존 버전 2에 관한 기록이다.** 현재 버전 3의 결과와 구분하여 보존한다.

## 환경

- Rust: `rustc 1.100.0 (0ef1bc386 2026-11-09)`.
- 설치 대상: `x86_64-unknown-linux-gnu`, `wasm32-unknown-unknown`.
- SDK: `ae6485e3c14ecd1fd5497d170eae884c68d04a87`, `aidoku` 0.3.0.
- 공식 BookOrbit 소스 기준: `v3.8.0`, `450c18e2acf79ddd39b6dee022a523281448e5e6`.
- Rust CLI의 `--help`, `package --help`, `verify --help`, `serve --help`, `build --help`를 실제 실행했다. CLI에는 `--version` 옵션이 없으며 현재 설치 바이너리의 정확한 빌드 revision은 확인하지 못했다. 재현 빌드용 설치 명령은 README에서 SDK revision으로 고정했다.
- 초기에는 셸의 GitHub raw·crates.io DNS 조회가 실패했으나, 네트워크 접근이 복구된 후 `cargo fetch --locked`로 누락된 의존성을 모두 받았다. SDK revision과 Cargo.lock은 유지했다.

## 실행한 검사

| 검사 | 결과 | 범위와 근거 |
|---|---|---|
| `cargo fmt --check` | PASS | 전체 Rust 파일 형식 |
| `cargo check --locked` | PASS | 기본 호스트 대상 핵심 모듈 |
| `cargo test --locked` | PASS | Rust 31개: 클라이언트 29개, 리소스 2개. 실패 0개 |
| `python3 -m unittest discover -s tests -p 'test_*.py'` | PASS | probe 8개. 실제 소켓·서버 연결 없음 |
| 핵심 모듈의 `no_std` Wasm 검사 | PASS | `/tmp/bookorbit-core-check`의 임시 rlib가 실제 `src/{api,auth,config,error,models,reader}.rs`를 path로 참조하여 `cargo check --offline --target wasm32-unknown-unknown` 실행. SDK 어댑터·링크·패키징 검사가 아님 |
| 메타데이터 공식 스키마 | PASS | 로컬 SDK CLI의 `source.schema.json`, `settings.schema.json`으로 Python jsonschema 검증 |
| 아이콘 크기·불투명도 | PASS | 128×128, 전체 alpha 255. RGB PNG와 tRNS 부재는 Rust 테스트로도 확인 |
| `sh -n scripts/validate.sh` | PASS | 셸 문법 검사 |
| 자격 증명 없는 probe 실행 | SKIP 확인 | 환경 변수 제거 상태에서 종료 코드 2, 네트워크 호출 없음 |
| `cargo fetch --locked` | PASS | 초기 빌드를 막던 SDK 의존성 다운로드 완료 |
| `cargo build --locked --release --target wasm32-unknown-unknown` | PASS | 고정된 실제 SDK와 `source.rs`를 포함한 전체 release 빌드 |
| `aidoku package` | PASS | 루트 `package.aix` 생성, 내부 Wasm과 release 산출물의 바이트 일치 |
| `aidoku verify package.aix` | PASS | 최소 API 버전·필수 export·아이콘 크기/불투명도·source/settings 스키마 모두 yes |
| `aidoku build -o public -n 'BookOrbit Private Sources' package.aix` | PASS | 개인 소스 목록·패키지 사본·아이콘 생성 |
| `aidoku serve` HTTP 점검 | PASS | 임시 포트에서 index.min.json → 패키지·아이콘 다운로드 후 원본과 바이트 비교. 점검 후 프로세스 종료 |
| `sh scripts/validate.sh` | PASS | 형식 → 호스트 검사 → Rust/Python 테스트 → Wasm 빌드 → 패키징 → CLI 검증, 최종 종료 코드 0 |
| 사용자 BookOrbit 서버 | PASS (사용자 보고) | 사용자 본인의 실제 서버에서 연동 정상 확인. 개발 환경에서 서버 주소·API 키를 사용한 직접 점검은 미실행 |
| iPhone Aidoku | PASS (사용자 보고) | 실제 설치·서버 연동·CBZ/CBR 열람·페이지 순서 정상 확인 |
| iPad Aidoku | NOT REPORTED | 별도 기기 검증 보고 없음 |

초기 온라인 `cargo check` 및 오프라인 전체 검증은 의존성·DNS 문제로 중단됐었다. 네트워크 복구 후 같은 잠금 파일로 의존성을 받은 뒤 `sh scripts/validate.sh`를 끝까지 실행하여 **종료 코드 0**을 확인했다. SDK 또는 소스를 우회하는 가짜 구현을 사용하지 않았다.

`aidoku verify`는 없는 파일에 대해 오류 문구와 종료 코드 0을 함께 반환할 수 있다. 최종 검증에서는 실제 ZIP·필수 파일·Wasm 헤더를 먼저 검사하고 모든 CLI 검사 결과가 yes인지 확인했다.

## 생성 산출물

| 경로 | 내용 |
|---|---|
| `package.aix` | 최종 패키지, 178,384바이트 |
| `target/wasm32-unknown-unknown/release/bookorbit_aidoku_source.wasm` | 전체 SDK를 포함한 Wasm, 461,977바이트 |
| `public/index.json`, `public/index.min.json` | 개인 소스 목록 |
| `public/sources/ko.bookorbitprivate-v2.aix` | 목록에서 배포하는 패키지 사본 |
| `public/icons/ko.bookorbitprivate-v2.png` | 목록에서 배포하는 아이콘 |

이 검증에서 생성된 `package.aix`의 SHA-256:

```text
4bd65ea0aac4ccffbef922d340c516c13ae88b6aa686870638ea6ecd354fbf00
```

패키지는 `Payload/main.wasm`, `source.json`, `settings.json`, `icon.png`를 포함한다. 내부 Wasm과 release 파일, 배포된 패키지 사본은 모두 바이트가 일치한다. ZIP 타임스탬프 등에 따라 다시 패키징하면 SHA-256은 달라질 수 있다.

## 테스트가 확인한 동작

- 서버 루트·후행 슬래시·프록시 하위 경로·기본 포트·IPv6 정규화.
- 잘못된 URL·userinfo·쿼리·fragment·탐색 경로·헤더 삽입 거부.
- 단일 Bearer 헤더와 비밀 값의 Debug 마스킹.
- 검색어 보존·UTF-16 길이 제한·CBZ/CBR 및 present 필터·0 기반 변환·최대 offset.
- nullable 메타데이터·빈 목록·다음/마지막 페이지·잘못된 envelope 및 중복 ID.
- Book ID와 File ID 구분·소속 확인·primary/content 필터·안정 정렬.
- 첫/마지막 페이지 인덱스·0페이지 및 과도한 페이지 수 오류.
- 401/403/404/410/422/429/500·연결 실패·파싱 실패·리다이렉트 감지.
- 429 최대 2회 재시도·백오프·Retry-After 준수·상한 초과 시 중단.
- 설정의 group/items 구조·키 마스킹 설정·아이콘 규격.
- probe의 리다이렉트 거부·스트리밍·재시도·비밀 값을 포함하지 않는 오류·미설정 시 skip.

fixture는 공식 타입을 바탕으로 작성한 **합성 데이터**다. 실제 CBZ/CBR 압축 해제, 사용자 서버 응답, Aidoku 호스트의 인증 헤더 전달이나 이미지 재시도를 검증한 결과는 아니다.

## 사용자 실기기 확인

2026-11-17, 사용자가 실제 iPhone에 설치하여 BookOrbit 서버 연동과 CBZ/CBR 열람·페이지 순서가 정상이라고 보고했다. 이후 한 항목씩 안내한 기본 검증 8개에 대해 다음 결과를 받았다.

| 번호 | 항목 | 결과 및 사용자 보고 범위 |
|---|---|---|
| 1 | 제목 검색 | PASS: 전체/일부 제목, 결과 없는 검색, 검색어 지우기 절차에 대해 “검색 정상” 보고 |
| 2 | 목록 페이지네이션 | PASS: 추가 목록 불러오기와 마지막 목록에서 로딩 종료 절차에 대해 “페이지네이션 정상” 보고 |
| 3 | 앱 재시작 후 보관함 연결 | PASS: 재시작 후 설정 재입력 없이 같은 작품·챕터 열람 절차에 대해 “재시작 정상” 보고 |
| 4 | 전체 페이지 수와 마지막 페이지 | PASS: BookOrbit 웹 리더와 Aidoku의 페이지 수·마지막 이미지 대조를 CBZ/CBR 각각 안내했고 “마지막 페이지 정상” 보고 |
| 5 | 인증 오류 및 복구 | PASS: 임시 잘못된 키로 오류 확인 후 원래 키 복원 절차에 대해 “인증 오류 복구 정상” 보고 |
| 6 | 네트워크 오류 및 복구 | PASS: 연결 차단 시 요청 종료 및 재연결 후 검색·열람 복구 절차에 대해 “네트워크 오류/복구 정상” 보고 |
| 7 | 다운로드 후 오프라인 열람 | PASS: CBZ/CBR 다운로드 후 연결을 끊고 앱을 재실행하여 첫·중간·마지막 페이지 확인 절차에 대해 “오프라인 열람 정상” 보고 |
| 8 | Aidoku 내부 읽던 위치 유지 | PASS: 미완독 상태에서 “계속 읽기”가 정상적으로 이어진다고 보고. 완독 후에는 “모든 장 읽기”로 바뀌고 첫 페이지에서 시작한다고 설명 |

8번은 미완독 상태의 이어 읽기와 완독 후 다시 읽기를 구분한다. 사용자 설명에 따른 완독 후 첫 페이지 시작은 이어 읽기 실패로 기록하지 않는다. 이 결과가 BookOrbit와의 읽기 진도 동기화를 의미하지는 않는다.

이 표는 **사용자 보고에 근거한 실기기 결과**다. 개발 환경에서 직접 기기·서버를 관찰하거나 응답·네트워크 로그를 수집한 것은 아니다. 서버·Aidoku·iOS의 정확한 버전은 제공되지 않았다. 기본 사용 검증은 8번까지로 마무리하며, 모든 압축 방식·이미지 형식·오류 시나리오를 확인한 것으로 확대하지 않는다.

## 남은 확인 항목

현재 빌드 차단 원인은 없다. 다만 다음 검증은 아직 수행하지 않았다.

- 사용자 서버 버전 및 실제 읽기 전용 API 키의 라이브러리 권한.
- iPad 등 다른 기기와 다른 서버·앱 버전의 호환성, 별도 권한 거부·손상 파일 등의 오류 시나리오.
- 호스트 이미지 429 처리와 리다이렉트 중 인증 헤더 제거.

개발 환경의 `BOOKORBIT_URL`과 `BOOKORBIT_API_KEY`는 설정되어 있지 않아 여기서 실제 서버 요청을 보내지 않았다. 실제 서버·아이폰의 기본 사용 검증 8개는 사용자 보고로 확인 완료했다. 위 항목은 추가 호환성·오류·보안 검증 범위로 구분하며 기본 사용 검증을 계속 늘리지 않는다. `scripts/probe_bookorbit.py`는 필요할 때 사용자가 별도로 실행할 수 있다.

## Phase별 결과

1. **감사 및 조사:** 초기 감사와 고정 버전 API/SDK 호환성 문서 작성 완료. 설치 서버 버전은 미확인.
2. **최소 연결:** 설정·인증·검색·상세 핵심 구현 및 모의 검증 완료. 사용자 실기기 보고로 실제 서버 연동·검색·페이지네이션 확인.
3. **리더:** 챕터·페이지 URL·인증 이미지 요청 어댑터 구현. 사용자 iPhone에서 CBZ/CBR 페이지 수·순서·마지막 페이지와 다운로드 후 오프라인 열람 정상 보고.
4. **품질 검증:** 형식·호스트 검사·Rust/Python 테스트·전체 Wasm release 빌드·패키징·CLI 검증·로컬 배포 경로 확인 완료.
5. **문서화:** 한국어 README·설치 절차·보안 경계·개발 가이드·직접 실행 결과·사용자 실기기 보고 반영 완료. 기본 설치·연동·열람 및 안내한 실기기 검증 8개를 사용자 보고로 확인했다.

기본 실기기 검증 8개는 사용자 보고로 마무리했다. 호스트 이미지 429 처리와 cross-origin 리다이렉트의 인증 헤더 제거는 별도의 추가 검증 범위다. 이 기록 이후 버전 3에서 시리즈 묶기를 추가했다. BookOrbit 진도 동기화는 후속 범위다.

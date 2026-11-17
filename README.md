# BookOrbit for Aidoku

개인 BookOrbit 서버의 CBZ/CBR/CB7 만화를 Aidoku에서 탐색하고 읽기 위한 Rust 소스입니다. BookOrbit 라이브러리를 Aidoku의 탐색 목록으로 제공하고, 같은 시리즈의 CBZ/CBR/CB7 도서를 작품 하나의 챕터들로 묶습니다. 시리즈가 없는 도서는 기존처럼 개별 작품으로 제공합니다.

**현재 상태:** 소스 버전 **4**에서 CB7을 추가했습니다. 기존 라이브러리별 탐색·시리즈 묶음에서도 CB7을 읽을 수 있습니다. Rust 테스트 45개·Python 테스트 9개, Wasm release 빌드, 패키징과 Aidoku CLI 검증을 통과했습니다. CB7과 라이브러리·시리즈 기능의 실제 서버·아이폰 검증은 아직 실행하지 않았습니다. 이전 버전의 기본 사용 검증 8개는 사용자 보고로 확인했으며, [검증 기록](docs/VALIDATION.md)에 버전별로 구분했습니다.

## 지원 기준과 기능

- BookOrbit 공식 **v3.8.0** 소스 계약 기준. API 키를 지원하는 서버가 필요하며 사용자 설치본 버전은 별도 확인해야 합니다.
- Aidoku 메타데이터 최소 버전 **0.8.3**. SDK의 timeout·최종 URL API 요구사항에 따른 값이며 실기기 호환 보증은 아닙니다.
- 서버 URL과 읽기 전용 API 키 설정, Bearer 헤더 인증.
- 서버 이름·표시 순서를 따르는 라이브러리별 목록과 전체 만화 목록.
- 만화 목록·제목 검색·40개 단위 페이지네이션·CBZ/CBR/CB7 필터. 검색은 전체 접근 가능 라이브러리 대상.
- BookOrbit 시리즈 ID 기반 묶음, 서버 권수 순서의 챕터, 40권을 넘는 시리즈의 전체 페이지 조회.
- 제목·저자·설명·장르/태그·출판사·언어·인증 표지.
- 도서/파일 ID 구분, 안정적인 챕터 키, 서버 페이지 순서에 따른 이미지 URL.
- REST 429 제한 재시도, 입력·응답·연결·인증·권한·손상 파일 오류 처리.

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
| `docs/` | [초기 감사](docs/INITIAL_AUDIT.md), [API 호환성](docs/API_COMPATIBILITY.md), [실행 결과](docs/VALIDATION.md) |

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

생성된 Wasm은 `target/wasm32-unknown-unknown/release/bookorbit_aidoku_source.wasm`, 패키지는 **프로젝트 루트의 [`package.aix`](package.aix)**입니다. 내부 Wasm이 release 빌드와 일치하는 것을 확인했습니다. 현재 산출물 크기와 SHA-256은 [검증 기록](docs/VALIDATION.md)에 적었습니다. 기본 호스트 `cargo check/test`와 별도로 전체 Wasm release 빌드도 통과했습니다.

CLI가 파일 읽기 실패에도 종료 코드 0을 반환하는 경우가 있어 스크립트는 실제 파일·ZIP 구성·Wasm 헤더를 먼저 검사합니다. `aidoku verify`는 설치 및 실제 네트워크 동작을 대신 검증하지 않습니다.

## Aidoku에 설치하기

이 체크아웃의 `package.aix`는 빌드·CLI 검증을 통과했습니다. 다음 절차로 기기에 설치하여 실제 서버 동작을 확인하세요.

1. 컴퓨터와 아이폰/아이패드를 같은 네트워크에 연결합니다. 컴퓨터의 8080 포트 접근을 허용합니다.
2. `aidoku serve package.aix`를 실행합니다. 서버를 실행한 터미널을 유지합니다.
3. CLI가 표시하는 `http://컴퓨터의-LAN-IP:8080/index.min.json`을 복사합니다. 아이폰에서 컴퓨터의 `localhost` 주소를 사용하지 마세요.
4. Aidoku의 소스 목록 관리에서 해당 URL을 추가하고 목록을 새로 고칩니다. 메뉴 이름은 앱 버전에 따라 다를 수 있습니다.
5. 표시된 **BookOrbit (Private)** 소스를 설치합니다.
6. 소스 설정에서 서버 기본 URL과 읽기 전용 API 키를 입력하고 목록을 다시 엽니다.

AirDrop 등으로 `.aix`를 기기에 전달해 Aidoku로 여는 방법도 SDK 문서에 안내되어 있습니다. 기기에서 파일 연결 메뉴가 제공되지 않으면 소스 목록 방식을 사용하세요.

개인 소스 목록도 `public/index.min.json`에 생성했습니다. 패키지는 `public/sources/ko.bookorbitprivate-v4.aix`, 아이콘은 `public/icons/ko.bookorbitprivate-v4.png`에 있습니다. 새 패키지로 목록을 갱신하려면 `sh scripts/build_source_list.sh`를 실행하세요. 패키지와 목록의 ID·버전·AIX 일치 여부를 검사합니다. `public/`은 생성물로 Git 추적에서 제외합니다. 패키지와 소스 목록에는 개인 URL/API 키가 들어가지 않습니다.

## GitHub Pages 소스 목록

Aidoku의 소스 목록 관리에서 다음 URL을 추가하고 새로 고침한 뒤 **BookOrbit (Private)**를 설치하세요. 설치 후 소스 설정에 개인 서버 URL과 읽기 전용 API 키를 입력합니다.

```text
https://10000je.github.io/aidoku-bookorbit-extension/index.min.json
```

GitHub Actions의 [`Deploy Aidoku source list`](.github/workflows/pages.yml)는 공개한 Release의 `package.aix`를 내려받아 고정된 공식 Aidoku CLI로 목록을 생성하고 GitHub Pages에 배포합니다. 배포되는 AIX는 Release 첨부 파일과 동일합니다. GitHub 저장소의 **Settings → Pages → Source**는 **GitHub Actions**로 설정합니다.

새 버전은 `res/source.json`의 정수 `version`을 올리고 전체 검증·패키징한 뒤, 변경을 커밋·푸시하고 새 정식 Release에 `package.aix`를 첨부해 게시하세요. 이 이벤트가 목록 배포를 자동 실행합니다. 초안과 사전 Release는 자동 배포하지 않습니다. 최신 정식 Release를 수동 재배포하려면 다음 명령을 사용합니다.

```sh
gh workflow run pages.yml
# 특정 Release를 배포하려면 태그를 지정합니다.
gh workflow run pages.yml -f release_tag=v4
```

배포 결과는 저장소의 **Actions**에서 확인할 수 있습니다. Pages에 올리는 내용은 소스 목록·확장 패키지·아이콘이며, BookOrbit 서버 자체나 서버의 도서는 게시하지 않습니다.

## 버전 4 업데이트와 표시 방식

소스 목록을 새로 고쳐 **버전 4**로 업데이트한 뒤 BookOrbit 탐색 화면을 다시 여세요. 기존 소스 ID와 서버/API 키 설정은 유지합니다. `전체 만화`와 접근 가능한 BookOrbit 라이브러리 이름이 나타납니다. 이 목록은 소스 탐색 구분이며 Aidoku 보관함의 사용자 카테고리를 자동 생성하는 기능은 아닙니다.

버전 4에서는 CB7도 CBZ/CBR과 같은 목록·시리즈·챕터·페이지 경로로 읽습니다. BookOrbit의 기존 `/cbz/files/{fileId}/pages` API가 7z 압축 해제와 페이지 이미지 제공을 담당합니다. 일반 `.7z` 파일을 CB7로 취급하지는 않습니다.

BookOrbit 웹에서 같은 시리즈인 `드래곤 c01`, `드래곤 c02`는 Aidoku에서 `드래곤` 한 작품으로 나타나고, 내부에 `Ch.1 드래곤 c01`, `Ch.2 드래곤 c02`가 생깁니다. 실제 화면 순서는 Aidoku의 챕터 정렬 설정에 따릅니다. 서버의 `seriesIndex` 순서를 유지하고, 읽을 수 있는 파일에 1부터 순번을 붙입니다. 권수가 빠졌거나 소수점 권수·외전이 있어도 제목은 원래 도서 제목을 유지합니다. 도서 하나에 만화 파일이 여러 개면 각각 챕터로 제공합니다.

시리즈 작품 키는 `series:{libraryId}:{seriesId}`, 챕터 키는 `{bookId}:{fileId}`입니다. 대표 도서가 달라져도 작품 키가 유지되고, 다른 라이브러리의 시리즈는 분리됩니다. **기존에 보관함에 저장한 개별 도서와 숫자 파일 키는 계속 열 수 있지만, 새 시리즈 작품으로 자동 병합하거나 읽기 기록·다운로드를 옮기지는 않습니다.** 기존 기록이 필요하면 개별 도서를 유지하면서 새 시리즈를 별도로 추가하세요.

## EPUB 텍스트 소설

현재 EPUB이 목록에 나타나지 않는 직접적인 이유는 확장이 목록과 챕터를 CBZ/CBR/CB7로 제한하기 때문입니다. Aidoku 자체에는 텍스트 리더가 있고, 고정 SDK도 `PageContent::Text`로 Markdown 텍스트를 전달할 수 있습니다. 그러므로 앱이 텍스트를 전혀 지원하지 않는다는 뜻은 아닙니다.

다만 EPUB 파일 자체의 열람은 별도 문제입니다. 고정 SDK의 `PageContent`는 이미지 URL·텍스트·이미지·ZIP 내부 이미지를 지원하고 EPUB 전용 콘텐츠 형식은 제공하지 않습니다. BookOrbit의 EPUB 목차/본문을 읽어 텍스트 페이지로 변환하는 연동은 아직 구현하지 않았습니다. EPUB 필터만 해제하면 읽을 수 없는 도서가 표시되므로 이번 변경에서도 제외합니다.

확인일 2026-11-17 기준 [Aidoku 공식 릴리스 노트](https://aidoku.app/release-notes/)는 텍스트 리더 지원을 설명하며, 별도 [EPUB 리더 PR #1052](https://github.com/Aidoku/Aidoku/pull/1052)는 열려 있습니다. 이 PR은 로컬 파일·Komga·Kavita 대상이므로 BookOrbit Wasm 확장의 지원으로 간주하지 않습니다. 설치된 앱의 정확한 버전과 EPUB 실기기 동작은 확인하지 않았습니다.

## 서버 설정과 인증

BookOrbit의 `Settings > You > API keys`에서 읽기 전용 키를 만들고 사용할 라이브러리만 지정하세요. 일반적인 탐색·페이지 읽기에 관리자·편집·다운로드 권한은 필요하지 않습니다. 서버에서 API 키 사용이 활성화되어 있고 사용자에게 키 생성 권한이 있어야 합니다.

- 서버 주소: `https://books.example.com` 또는 `https://books.example.com/shelf`처럼 서버 기본 URL을 입력합니다. 후행 슬래시는 정리합니다.
- `/api/v1`, `/opds`, 쿼리, fragment, URL 내 계정/비밀번호를 넣지 마세요.
- 최종 HTTPS 주소를 권장합니다. HTTP는 개인 LAN 등에서 의도적으로 사용할 때만 선택하세요.
- API 키 입력은 마스킹되며 설정 저장은 Aidoku가 관리합니다. 키를 Git·로그·오류 메시지·이미지 URL에 넣지 않습니다.
- OPDS Basic 계정은 이 소스의 REST API 키로 사용할 수 없습니다.

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

## 아이폰 기본 사용 검증

아래 8개는 사용자 실기기 보고로 확인 완료했습니다. 구체적인 절차와 보고 범위는 [검증 기록](docs/VALIDATION.md)의 사용자 실기기 확인 표를 참고하세요.

1. 전체/일부 제목 검색, 결과 없는 검색, 검색어 지우기.
2. 목록 추가 불러오기와 마지막 목록에서 로딩 종료.
3. 앱 완전 종료·재실행 후 같은 보관함 작품·챕터 열람.
4. CBZ/CBR의 웹 리더 대비 전체 페이지 수와 마지막 이미지 대조.
5. 잘못된 API 키의 오류 표시와 원래 키 복원 후 정상 사용.
6. 네트워크 차단 시 오류 표시와 재연결 후 정상 사용.
7. CBZ/CBR 다운로드 후 네트워크를 끊고 앱을 재실행하여 오프라인 열람.
8. 미완독 상태에서 “계속 읽기”로 읽던 위치 이어 읽기.

사용자 설명에 따르면 완독 후에는 “모든 장 읽기”로 바뀌어 첫 페이지에서 시작합니다. 이 다시 읽기 동작은 미완독 상태의 이어 읽기 실패와 구분합니다. BookOrbit와의 진도 동기화는 별도 후속 범위입니다.

## 알려진 제한사항

- 사용자 보고로 아이폰의 기본 사용 검증 8개를 확인했습니다. 서버·앱·iOS의 정확한 버전과 iPad 등 다른 기기의 결과는 보고되지 않았습니다. 별도 권한 거부·손상 파일 등의 오류 시나리오 전체를 확인한 것은 아닙니다.
- SDK에는 리다이렉트 차단 옵션이 없습니다. REST 최종 URL 검사는 사후 검사입니다. 호스트가 자동 이동 중 인증을 제거하는지는 보장하지 않으므로 리다이렉트 없는 최종 URL을 사용하고 API 요청이 다른 호스트로 이동하지 않도록 구성해야 합니다.
- 이미지 다운로드 이후의 429 재시도·오류 표시는 Aidoku가 담당합니다. 소스에서 검증한 2회 재시도는 REST JSON 요청에 적용됩니다.
- 긴 Retry-After, HTTP-date 또는 잘못된 값은 자동 재시도하지 않습니다. JSON은 수신 후 4 MiB 제한을 검사하며 파일당 최대 20,000페이지입니다.
- CBR/CB7 암호화·손상·서버 추출 실패, 특수 이미지 형식은 서버와 iOS 디코더의 지원에 따릅니다.
- ID는 같은 서버 안에서 유지됩니다. 서버를 교체하면 같은 숫자 ID가 다른 도서를 뜻할 수 있습니다.
- 라이브러리가 전자책만 포함하면 해당 목록은 비어 있습니다. EPUB/PDF/오디오 형식은 이 확장의 열람 대상이 아닙니다.
- 시리즈 정보가 없는 도서를 제목/파일명의 유사성만으로 묶지는 않습니다. 시리즈 상세·표지·저자 등은 첫 번째 읽을 수 있는 만화 도서에서 가져옵니다.
- 시리즈당 최대 20,000권입니다. 조회 중 권수·시리즈 정보가 바뀌거나 일부 페이지가 실패하면 일부 권만 반환하지 않고 오류를 표시합니다.
- 최근 추가·BookOrbit 진도 동기화·OPDS fallback은 후속 범위입니다.

## Git 관리

로컬 Git 저장소의 기본 브랜치는 `main`입니다. 첫 커밋은 기존 버전 3 전체를 보존하고, 이후 CB7 변경을 별도 커밋으로 관리합니다. 원격 저장소는 아직 연결하지 않았습니다.

소스·설정 스키마·아이콘·합성 fixture·문서·`Cargo.lock`은 추적합니다. `target/`, `package.aix`, `public/`, Python 캐시와 `.env` 파일은 `.gitignore`로 제외합니다. 설치 패키지와 소스 목록은 위 빌드 명령으로 다시 생성합니다.

## 공식 참고

- [Aidoku Rust SDK/CLI](https://github.com/Aidoku/aidoku-rs), [개발 문서](https://aidoku.github.io/aidoku-rs/)
- [BookOrbit](https://github.com/bookorbit/bookorbit), [API 키](https://github.com/bookorbit/bookorbit/blob/main/docs/API_KEYS.md), [OPDS](https://bookorbit.app/opds)

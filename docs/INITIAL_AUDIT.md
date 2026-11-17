# 초기 감사

검토일: 2026-11-17. 최초 입력물은 `src/lib.rs`, `Cargo.toml`, `res/` 및 한국어 README로 구성된 미검증 단일 파일 구현이었다. Git 이력·테스트·CI·잠금 파일은 없었다. 별도 `BookOrbit-Aidoku-Source` 디렉터리는 없어 현재 루트를 출발점으로 사용했다.

## 확인된 기존 기능

- 설정에서 URL/API 키 읽기, `X-Api-Key` 인증, 검색·페이지네이션과 CBZ/CBR 필터.
- 도서 ID → 작품, 파일 ID → 챕터, `pageCount` → 0 기반 이미지 URL.
- 이미지 요청용 `ImageRequestProvider`, 401·403·429의 사용자 메시지.
- 파일 다운로드 대신 페이지 URL만 생성하여 이미지 전체를 Wasm에 적재하지 않는 설계.

## 공식 코드와 대조

Aidoku SDK의 로컬 공식 체크아웃 `ae6485e3c14ecd1fd5497d170eae884c68d04a87`에서 `Source`, `ImageRequestProvider`, `defaults_get`, `Request`, `Manga`, `Chapter`, `PageContent::url`을 확인했다. 기존 코드의 해당 인터페이스가 허구인 것은 아니다. 설정의 `text`·`secure` 속성도 CLI 스키마에 존재한다.

BookOrbit 공식 `v3.8.0` 소스 `450c18e2acf79ddd39b6dee022a523281448e5e6`과 웹에서 열람한 main 컨트롤러·타입을 비교했다. 검색 및 페이지 API는 실제 존재한다. 목록의 저자는 문자열 배열, 상세의 저자는 객체 배열이므로 기존 구분은 올바르다. 사용자 서버의 버전은 아직 확인되지 않았다.

## 수정해야 할 문제

| 영역 | 초기 상태와 영향 | 조치 |
|---|---|---|
| 재현성 | SDK main 추적, Cargo.lock 없음 | SDK revision 및 의존성 잠금 |
| URL | 문자열 접두사 검사, userinfo·쿼리·탐색 경로 허용 | URL 파서로 origin·프록시 경로·이미지 라우트 검사 |
| 페이지네이션 | 음수 페이지가 큰 양수로 변환 | 유효 범위 검사, 서버의 최대 offset 반영 |
| 파일 | 역할 무시, 서버 응답 순서에 챕터 번호 의존 | primary/content만 사용, 파일 ID 기준 안정 정렬 |
| 응답 | 상세 ID 검증 없음, 누락 files가 빈 결과로 은폐 | 식별자·필수 필드·페이지 수·목록 envelope 검증 |
| 오류 | 429 즉시 실패, timeout 없음 | 제한 재시도·백오프·timeout·안전한 오류 분류 |
| 호환성 | settings의 최상위 group 누락, 256×256 아이콘, 최소 Aidoku 버전·listing 없음 | group/items 구조, 128×128 불투명 아이콘, 최소 버전·listing 보완 |
| 검증 | 테스트·실측 근거 없음 | 합성 fixture, mock transport, 별도 opt-in 서버 점검 |

## 미구현 및 범위

시리즈 묶기, OPDS Basic, 진도 동기화는 후속 범위다. 초기 구현의 정상적인 도서/파일 ID와 읽기 흐름은 보존한다. 호스트가 수행하는 이미지 다운로드의 재시도·리다이렉트는 SDK의 제어 범위를 별도로 확인하며, 컴파일 성공을 실기기 성공으로 취급하지 않는다.

## 실행 환경

Rust 1.100.0, Wasm 대상, Rust Aidoku CLI는 설치되어 있다. 초기 `cargo check --offline`은 `cobs` 등 미캐시 의존성 때문에, 온라인 `cargo check`는 `static.crates.io` DNS 실패 때문에 중단됐다. 이 단계에서는 기존 코드가 컴파일된다고 판단할 수 없다. 최종 실행 결과는 `docs/VALIDATION.md`에 별도로 기록한다.

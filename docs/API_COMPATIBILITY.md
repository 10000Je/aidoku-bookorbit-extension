# API 및 SDK 호환성

검토일: 2026-11-17. 사용자 서버의 버전과 Aidoku 앱 버전은 미확인이다. 아래 계약은 공식 소스 코드로 확인했으며 실제 서버 응답을 수집한 결과가 아니다. 이후 사용자 보고로 실제 iPhone 설치·서버 연동과 검색·페이지네이션·재시작·페이지 수/마지막 페이지·인증/네트워크 오류 복구·오프라인 열람·미완독 이어 읽기의 기본 검증 8개를 확인했다. 각 결과의 범위는 [검증 기록](VALIDATION.md)에 구분한다.

## 기준 버전과 근거

- BookOrbit: [`v3.8.0`, `450c18e`](https://github.com/bookorbit/bookorbit/tree/450c18e2acf79ddd39b6dee022a523281448e5e6). 작업 공간에 보존된 공식 체크아웃의 컨트롤러·서비스·타입·키 정책을 직접 열람했다.
- Aidoku Rust SDK: [`ae6485e`](https://github.com/Aidoku/aidoku-rs/tree/ae6485e3c14ecd1fd5497d170eae884c68d04a87), `aidoku` 0.3.0. `Cargo.toml`에서 revision을 고정하고 `Cargo.lock`을 유지한다. 이 revision이 원격 main의 최신 HEAD라고 단정하지 않는다.
- CLI: 같은 Rust 저장소의 `aidoku-cli` 0.1.0 소스와 설치된 `package`, `verify`, `serve`, `build`의 `--help`를 확인했다. 설치 바이너리의 정확한 revision은 미확인이며 README의 재설치 명령은 SDK revision으로 고정했다. 별도 레거시 CLI 명령을 사용하지 않는다.
- 웹에서 BookOrbit main의 [도서 컨트롤러](https://github.com/bookorbit/bookorbit/blob/main/server/src/modules/book/book.controller.ts), [타입](https://github.com/bookorbit/bookorbit/blob/main/packages/types/src/book.ts), [키 문서](https://github.com/bookorbit/bookorbit/blob/main/docs/API_KEYS.md)도 보조 확인했다. 호환 기준은 위 고정 태그다.

## REST 계약

모든 경로는 사용자 기본 URL 뒤에 `/api/v1`을 붙인다. `https://host/shelf/`는 `https://host/shelf/api/v1/...`가 된다. 서버의 prefix는 `server/src/main.ts`에 정의되어 있다.

| 기능 | 메서드와 경로 | 사용하는 계약 | 공식 근거 경로 |
|---|---|---|---|
| 서버 정보 | GET `/app-info` | `version` | `server/src/modules/app-info/` |
| 인증 점검 | GET `/auth/me` | `authenticationMethod: api_key` | `server/src/modules/auth/auth.controller.ts`, `docs/API_KEYS.md` |
| 목록/검색 | POST `/books/query` | BookQuery → BooksPage | `server/src/modules/book/book.controller.ts:globalQuery`, `packages/types/src/query.ts`, `book.ts` |
| 라이브러리 목록 | GET `/libraries` | 접근 가능한 Library 배열, `id`, `name`, `displayOrder` | `server/src/modules/library/library.controller.ts`, `library.service.ts`, `packages/types/src/library.ts` |
| 라이브러리 도서 | POST `/libraries/{libraryId}/books` | BookQuery → BooksPage, viewer 권한, 읽기 전용 검색 | 같은 컨트롤러의 `queryBooks` |
| 시리즈 도서 | GET `/series/{seriesId}/books?libraryId={libraryId}&page=0&size=40&sort=seriesIndex&order=asc` | SeriesBooksPage, `seriesInfo`, contextual seriesId | `server/src/modules/series/series.controller.ts`, `series.service.ts`, `dto/list-series-books.dto.ts`, `packages/types/src/series.ts` |
| 상세 | GET `/books/{bookId}` | BookDetail, `files[]` | 같은 컨트롤러, `packages/types/src/book.ts` |
| 표지 | GET `/books/{bookId}/thumbnail`, `/cover` | 이미지 응답, 없으면 404 | 같은 컨트롤러의 `getThumbnail`, `getCover` |
| 페이지 수 | GET `/cbz/files/{fileId}/pages` | `{ "pageCount": N }` | `server/src/modules/reader/cbz/cbz.controller.ts:getPageCount` |
| 이미지 | GET `/cbz/files/{fileId}/pages/{index}` | 서버가 선택한 이미지 스트림 | 같은 컨트롤러의 `getPage`, `cbz.service.ts:streamPage` |

위 근거 경로는 모두 고정된 [BookOrbit 소스 트리](https://github.com/bookorbit/bookorbit/tree/450c18e2acf79ddd39b6dee022a523281448e5e6)에 상대적이다. 서버가 `SWAGGER_ENABLED=true`로 운영되면 `/api/docs` 및 `/api/docs-json`에서도 해당 설치본의 계약을 확인할 수 있다. 이번 작업에서는 사용자 OpenAPI에 접근하지 않았다.

### 인증과 권한

[공식 API 키 문서](https://github.com/bookorbit/bookorbit/blob/450c18e2acf79ddd39b6dee022a523281448e5e6/docs/API_KEYS.md)는 `Authorization: Bearer`와 `X-Api-Key`를 모두 지원한다. 이 소스는 **Bearer 하나만** 전송한다. 기존 키 설정 이름 `api_key`는 그대로 유지한다. 다른 세션 토큰과 중복 전송하지 않는다. URL 쿼리로 키를 전달하지 않는다.

`globalQuery`의 `@KeyReadSafe()` 때문에 읽기 전용 키로 POST 검색이 가능하다. 서비스는 현재 사용자의 접근 가능 라이브러리·콘텐츠 필터를 적용한다. `CbzService.openComic`은 `BookService.verifyFileAccess`를 호출해 이미지 접근도 검사한다. 클라이언트는 서버의 접근 통제를 우회하는 별도 목록을 만들지 않는다. 다운로드·메타데이터 편집·진도 쓰기 권한은 요청하지 않는다.

### 검색과 페이지네이션

```json
{
  "pagination": {"page": 0, "size": 40},
  "sort": [{"field": "title", "dir": "asc"}],
  "collapseSeries": true,
  "collapseLinked": false,
  "filter": {"type": "group", "join": "AND", "rules": [
    {"type": "rule", "field": "format", "operator": "includesAny", "value": ["cbz", "cbr", "cb7"]},
    {"type": "rule", "field": "fileAvailability", "operator": "isPresent"}
  ]},
  "q": "검색어"
}
```

`BookQueryPipe`에서 page는 0부터, size는 1~200, 검색어는 JavaScript 문자열 길이 기준 200 이하다. 이 소스는 Aidoku의 1 기반 page를 검증 후 변환하고 검색어의 UTF-16 길이를 검사한다. `pagination.constants.ts`의 최대 offset 1,000,000도 적용한다. 응답의 `items`, `total`, `page`, `size`를 검사한다. 다음 페이지 판단에는 필터 후 항목 개수가 아닌 서버 total과 offset을 사용한다.

목록 저자는 `string[]`, 상세 저자는 `{id,name,sortName}[]`다. 제목·설명은 null일 수 있다. `files`에는 실제 만화 외에 표지·sidecar·전자책이 있으므로 `role`이 primary/content이고 format이 CBZ/CBR/CB7인 파일만 챕터로 사용한다. 도서 ID와 파일 ID는 별도 타입이며, 기존 개별 도서의 작품·챕터 키는 숫자 문자열을 보존한다. 새 시리즈 작품은 `series:{libraryId}:{seriesId}`, 해당 챕터는 `{bookId}:{fileId}`를 사용한다.

### 라이브러리와 시리즈 묶음 — 소스 버전 3

`GET /libraries` 응답을 `displayOrder`, ID 순으로 정렬하고 이름을 그대로 사용한다. SDK `DynamicListings`가 `library:{id}` 목록을 반환하고 `ListingProvider`가 `POST /libraries/{id}/books`로 조회한다. 기존 `comics` 목록은 `전체 만화`로 유지한다. 이 탐색 목록은 Aidoku 보관함 카테고리를 생성하지 않는다. 전체 제목 검색은 기존 `POST /books/query`를 사용한다.

소스 버전 5부터 `DynamicListings`는 각 라이브러리를 기존 검색 계약으로 조회하여 표시 가능한 만화가 있는 곳만 반환한다. 검색과 동일하게 `present` 도서의 primary/content CBZ·CBR·CB7 파일을 판정한다. 첫 페이지에 표시할 항목이 없고 다음 페이지가 있으면 계속 조회하며, 첫 항목을 찾으면 해당 라이브러리 확인을 마친다. 모두 빈 결과이면 버튼을 숨긴다. 서버 이름·표시 순서·ID를 유지하고, 조회 실패는 빈 결과로 바꾸지 않고 기존 오류로 전달한다. 새 API나 쓰기 요청은 추가하지 않는다. 표시 이름을 `BookOrbit`으로 바꾸되 기존 설치·설정·보관함 연결을 위해 소스 ID `ko.bookorbitprivate`를 유지한다.

도서 목록·검색은 `collapseSeries: true`, `collapseLinked: false`로 요청한다. 서버 `book.repository.ts`의 `COLLAPSE_GROUP_KEY_SQL`은 라이브러리와 시리즈 ID로 묶으므로 클라이언트도 같은 식별 기준을 사용한다. `total`은 묶인 목록 항목 수이며 클라이언트에서 한 페이지씩 임의로 묶지 않는다. 시리즈명이 같은 다른 라이브러리도 구분한다. 제목 문자열에서 시리즈를 추측하지 않는다.

시리즈 상세에서는 `GET /series/{id}/books`를 라이브러리 ID와 함께 요청한다. 40권 단위로 모든 페이지를 수집한 뒤 primary/content CBZ/CBR/CB7만 챕터로 만든다. API의 `sort=seriesIndex&order=asc`를 사용하므로 공식 `series-index.ts`의 소수점/정수 부분 비교 규칙도 서버에 맡긴다. Aidoku 챕터 번호는 읽을 수 있는 파일 순서의 1 기반 순번이며 원래 권수 표기는 도서 제목에 유지된다. 같은 도서의 여러 파일만 파일 ID로 정렬한다.

시리즈 API는 요청된 시리즈의 membership으로 `seriesId`를 문맥화한다. 파일 열람 전 상세의 `libraryId`와 기본 `seriesId` 또는 `seriesMemberships[]`를 검사하고, 해당 도서의 CBZ/CBR/CB7 파일인지 확인한다. 서버의 접근 통제도 그대로 적용된다. 페이지별 ID·라이브러리·시리즈·전체 건수·이름 일관성과 전체 수집 건수를 확인한다. 20,000권 초과, 중복·누락·부분 실패는 오류이며 일부 권만 정상 시리즈로 반환하지 않는다.

기존 보관함의 숫자 도서/파일 키는 계속 처리한다. 새 시리즈 키와의 자동 병합·진도 이전은 구현하지 않으며 `breakingChangeVersion`도 설정하지 않는다. 시리즈 메타데이터는 첫 만화 도서 상세를 사용한다.

### 페이지와 CBR/CB7

`cbz.service.ts`는 CBZ·CBR·CB7을 모두 처리하며 CBR은 서버의 `node-unrar-js`로 페이지를 추출한다. CB7은 `getSevenZPages` → `cb7-disk-extraction.ts`의 `extractCb7ToDisk`에서 서버 임시 디렉터리로 추출하고 이미지 목록을 정렬한다. 동일한 `/cbz/files/{fileId}/pages`와 `/cbz/files/{fileId}/pages/{index}`로 읽으므로 확장에 새 압축 해제 라이브러리나 다른 이미지 경로를 추가하지 않는다. 소스 버전 4에서 CB7을 목록·검색 필터와 primary/content 판정, 읽기 전용 probe에 추가했다. 대소문자를 구분하지 않으며 일반 `7z` 형식은 허용하지 않는다. 이미지 이름은 서버의 `compareArchiveEntryNames`로 정렬된다. 클라이언트는 파일명을 재정렬하지 않고 `0`부터 `N-1`까지 생성한다. 0페이지는 명확한 오류, 20,000 초과는 안전 한도 오류로 반환한다.

서버는 페이지별 MIME, ETag, `private, no-cache`를 제공한다. 클라이언트는 이미지 URL만 Aidoku에 전달하며 전체 아카이브나 모든 이미지 바이트를 Wasm에 적재하지 않는다. 사용자 iPhone에서 CBZ/CBR 열람과 페이지 순서가 정상이라는 보고를 받았다. CB7의 실제 서버 추출·iPhone 열람은 아직 확인하지 않았다. 개별 압축 방식·이미지 형식 전체에 대한 호환성을 확인한 것은 아니다.

## Aidoku 인터페이스

[고정 SDK](https://github.com/Aidoku/aidoku-rs/tree/ae6485e3c14ecd1fd5497d170eae884c68d04a87)의 `crates/lib/src/structs/source.rs`에서 `Source`, `ListingProvider`, `DynamicListings`, `ImageRequestProvider`를 확인했다. 이미지 trait은 표지·썸네일·페이지 요청을 대체하는 기능이다. `imports/defaults.rs`의 `defaults_get`, `imports/net.rs`의 `Request::header`, `timeout`, `Response::get_url`, `get_data`, `imports/std.rs`의 `sleep`을 사용한다.

SDK는 Wasm에서 allocator와 panic handler를 제공한다. 호스트 테스트에서는 SDK의 FFI를 호출하지 않고 같은 핵심 모듈을 std 테스트로 실행한다. 이후 `source.rs`를 포함한 전체 Wasm release 빌드와 `.aix`의 CLI 검증도 별도로 통과했다. 사용자 실기기 보고로 iPhone에서의 소스 실행·서버 연동·CBZ/CBR 열람을 확인했다. 직접 수집한 기기 로그나 서버 응답은 없다. 결과는 [검증 기록](VALIDATION.md)에 구분한다.

CLI의 `settings.schema.json`은 최상위 `group` 배열과 내부 `items`를 요구한다. 초기 평면 배열을 수정했다. 아이콘도 CLI 규격인 128×128 불투명 PNG로 맞췄다. 사용한 `get_url`, `set_timeout`은 CLI 검증기의 최소 앱 버전 0.8.3 조건을 따르므로 메타데이터에 `minAppVersion: 0.8.3`을 명시했다. 실제 설치 앱에서 확인해야 한다.

## 오류·재시도·보안 경계

- REST의 401/403/404/410/422/429, 연결 실패 및 잘못된 JSON을 구분한다. 외부 오류 본문은 사용자 메시지에 복사하지 않는다.
- REST 429만 최대 2회 재시도한다. 기본 대기는 1초·2초다. 정수 Retry-After는 최대 10초까지 지키며, 긴 값·HTTP-date·잘못된 값이면 조기 재시도하지 않고 오류를 반환한다. 요청별 idle timeout은 20초다.
- JSON은 4 MiB 한도를 검사한다. SDK가 본문을 받은 후 검사하므로 스트리밍 수신 크기 제한을 보장하지 않는다.
- 이미지 최초 요청은 파싱한 origin·프록시 mount·허용 이미지 경로를 모두 검사한다. 동시 요청 수는 2다.
- **SDK Request에는 리다이렉트 차단 옵션이 없다.** REST 최종 URL이 바뀌면 오류로 반환하지만 이는 사후 검사다. 호스트의 자동 리다이렉트 중 인증 제거 여부를 여기서 보장할 수 없다. 최종 HTTPS 기본 URL을 설정하고 프록시가 API 요청을 다른 origin으로 이동시키지 않도록 해야 한다. 실기기 네트워크 검사 전에는 이 보안 항목을 검증 완료로 취급하지 않는다.
- 이미지 다운로드 이후의 429 재시도·오류 표시는 Aidoku 호스트가 담당한다. `ImageRequestProvider`는 응답 콜백이 아니므로 이 소스의 REST 재시도와 동일하다고 주장하지 않는다. 호스트 이미지 재시도와 리다이렉트는 남은 검증 항목이다.

## REST와 OPDS 비교

[OPDS 공식 문서](https://bookorbit.app/opds)에 따르면 OPDS는 별도 계정의 Basic 인증과 `opds_access` 권한을 사용하며 Atom 목록·표지·primary 파일 다운로드를 제공한다. REST API 키의 권한 체계와 다르다. MVP는 파일 ID와 서버 페이지 API를 직접 사용할 수 있는 REST를 선택했다. API 키를 지원하지 않는 서버의 OPDS Basic 연동은 후속 구현이며 자동 fallback하지 않는다.

## EPUB 조사 — 2026-11-17

현재 목록/파일 필터는 CBZ/CBR/CB7만 허용한다. 고정 SDK `crates/lib/src/structs/mod.rs`의 `PageContent::Text`는 Markdown 텍스트를 지원하므로 텍스트 소설 전체가 앱의 원천적 비지원 대상인 것은 아니다. 그러나 EPUB 콘텐츠 전용 variant는 없으며 BookOrbit EPUB 본문/목차→텍스트 페이지 변환은 미구현이다. [공식 릴리스 노트](https://aidoku.app/release-notes/)에서도 텍스트 리더를 확인했다. [EPUB 리더 PR #1052](https://github.com/Aidoku/Aidoku/pull/1052)는 확인 시 열려 있고 로컬 파일·Komga·Kavita 연동을 대상으로 한다. 앱의 EPUB 전용 리더와 이 Wasm 확장의 지원을 동일하게 취급하지 않는다. 이번 버전은 EPUB 표시/열람을 추가하지 않는다.

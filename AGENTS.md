# Repository Guidelines

## 프로젝트 구조

BookOrbit의 CBZ/CBR 도서를 Aidoku에서 읽기 위한 Rust 2024·WebAssembly 소스입니다. `src/`는 설정, 인증, API 모델, 데이터 변환 및 Aidoku 인터페이스를 담당합니다. `res/source.json`은 소스 메타데이터, `res/settings.json`은 사용자 설정, `res/icon.png`는 아이콘입니다. 테스트와 fixture는 실제 서버 없이 실행할 수 있도록 구성합니다. `docs/`에는 초기 감사, API 호환성 및 검증 결과를 기록합니다.

## 빌드 및 검증

프로젝트 루트에서 다음 명령을 실행합니다.

```sh
cargo fmt --check
cargo check
cargo test
cargo build --release --target wasm32-unknown-unknown
aidoku package
aidoku verify package.aix
```

Wasm 대상이 없다면 `rustup target add wasm32-unknown-unknown`으로 설치합니다. CLI 명령은 설치된 버전의 `aidoku --help`와 하위 명령 도움말로 확인합니다. `aidoku serve package.aix`는 개인용 소스 목록을 로컬 네트워크에 제공합니다. 패키지 생성 위치는 실제 CLI 출력으로 확인합니다.

## 코드 스타일 및 설계

Rust는 rustfmt와 공백 네 칸을 사용합니다. 함수·필드는 `snake_case`, 타입은 `PascalCase`, 상수는 `SCREAMING_SNAKE_CASE`로 작성합니다. JSON은 공백 두 칸을 사용합니다. Wasm의 `no_std` 제약을 유지하고 네트워크 요청과 순수 데이터 변환을 분리합니다. 외부 입력에 무분별한 `unwrap()`을 사용하지 않습니다. 도서 ID와 파일 ID를 구분하고 안정적인 작품·챕터 식별자를 유지합니다.

## 테스트 원칙

URL 정규화, 인증 헤더, 검색·페이지네이션, 응답 파싱, ID 매핑, 페이지 순서 및 오류 처리를 검증합니다. 테스트 이름은 확인하는 동작을 설명합니다. 실제 서버 테스트는 별도로 실행하며 자격 증명이 없으면 자동 실행하지 않습니다. 실행하지 못한 검증은 통과로 보고하지 않습니다. 컴파일, 패키지 검증, 아이폰 실기기 확인 결과를 구분합니다.

## 커밋 및 PR

현재 체크아웃에는 확인 가능한 Git 이력이 없습니다. 커밋 제목은 `Fix chapter ordering`처럼 간결한 명령형을 권장합니다. PR에는 변경 목적, 동작 차이, 실행한 검증, 호환 SDK·서버 버전 및 남은 제약을 적습니다. 관련 이슈를 연결하고 설정 UI 변경에는 필요한 화면 자료를 첨부합니다.

## 보안 및 작업 규칙

공식 SDK와 서버 코드로 확인한 인터페이스만 사용합니다. API 키는 설정 또는 로컬 환경 변수로만 제공하고 코드, URL 쿼리, 로그 및 오류 메시지에 넣지 않습니다. 읽기 전용 권한을 사용하고 다른 호스트로 인증 정보를 전달하지 않습니다. 리다이렉트도 검토합니다. 사용자 서버의 데이터는 수정·삭제하지 않습니다. 문서와 작업 보고는 한국어로 작성합니다.

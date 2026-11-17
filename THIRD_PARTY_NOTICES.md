# 오픈소스 프로젝트 및 라이선스 고지

확인일: 2026-11-18.

## 프로젝트의 관계

**BookOrbit for Aidoku**는 Aidoku에서 개인 BookOrbit 서버를 이용하기 위해 독립적으로 개발·관리하는 비공식 확장 소스입니다. Aidoku Rust SDK로 만든 WebAssembly 확장이 Aidoku 안에서 실행되고, BookOrbit의 REST API를 통해 도서 정보와 만화 페이지를 읽습니다.

Aidoku 앱과 BookOrbit 서버는 각각 별도로 설치·운영합니다. 이 저장소에서 배포하는 AIX에는 확장 코드와 설정·아이콘이 들어갑니다. 두 원본 프로젝트의 이름은 연동 대상과 출처를 표시하며, 이 확장이 원본 개발팀의 공식 배포물이나 후원·승인 제품이라는 의미는 아닙니다.

## 원본 프로젝트와 라이선스

| 프로젝트 | 이 확장과의 관계 | 원본 라이선스 및 근거 |
|---|---|---|
| [Aidoku](https://github.com/Aidoku/Aidoku) | 확장을 실행하는 리더 앱 | 앱 코드: [GNU GPL v3](https://github.com/Aidoku/Aidoku/blob/95d1468d599a154f03d6a1dd06bf839d3ff3f7bd/LICENSE). 번역은 [원본 README](https://github.com/Aidoku/Aidoku/blob/95d1468d599a154f03d6a1dd06bf839d3ff3f7bd/README.md#translations)에서 별도 Apache 2.0으로 안내합니다. |
| [Aidoku Rust SDK / CLI](https://github.com/Aidoku/aidoku-rs) | Rust 바인딩과 AIX 패키징·소스 목록 생성 도구 | [MIT](https://github.com/Aidoku/aidoku-rs/blob/ae6485e3c14ecd1fd5497d170eae884c68d04a87/LICENSE), 사용 revision: `ae6485e3c14ecd1fd5497d170eae884c68d04a87`. |
| [BookOrbit](https://github.com/bookorbit/bookorbit) | 도서·라이브러리·시리즈·페이지를 제공하는 서버 | [GNU AGPL v3 only](https://github.com/bookorbit/bookorbit/blob/450c18e2acf79ddd39b6dee022a523281448e5e6/LICENSE), [NOTICE](https://github.com/bookorbit/bookorbit/blob/450c18e2acf79ddd39b6dee022a523281448e5e6/NOTICE), [추가 조건](https://github.com/bookorbit/bookorbit/blob/450c18e2acf79ddd39b6dee022a523281448e5e6/ADDITIONAL_TERMS.md). API 호환 기준: `v3.8.0` (`450c18e`). |

각 원본 프로젝트의 코드와 자료에 대한 저작권은 원저작자와 기여자에게 있습니다. 구체적인 API 연동 범위와 확인한 버전은 [호환성 문서](docs/API_COMPATIBILITY.md)에 기록합니다.

## Aidoku Rust SDK 고지

SDK의 저작권 표시는 `Copyright (c) 2022-2025 Skittyblock`입니다. 사용 revision의 MIT 라이선스 원문을 [licenses/aidoku-rs-MIT.txt](licenses/aidoku-rs-MIT.txt)에 보존합니다. 해당 SDK 코드가 포함된 배포물에는 이 저작권 및 허가 고지를 함께 유지해야 합니다.

## BookOrbit 출처 고지

[Powered by BookOrbit](https://github.com/bookorbit/bookorbit)

Copyright (C) 2025-2026 neon.
Additional copyright is held by the respective BookOrbit contributors.

BookOrbit의 NOTICE는 AGPL v3 only와 추가 조건을 함께 안내합니다. 추가 조건에는 적용 대상 BookOrbit 자료를 포함하는 저작물의 출처 표시, 수정본 표시, 고지 보존 및 상표 관련 조건이 있습니다. 적용 범위와 정확한 문구는 위 원본 문서를 확인하세요. 이 저장소는 BookOrbit REST API와 연동하며, 해당 서버의 동작을 확인한 버전을 출처와 함께 기록합니다.

## 이 저장소 자체의 라이선스

현재 이 저장소에는 자체 코드의 배포 라이선스를 지정하는 `LICENSE` 파일이 없습니다. 위 표는 원본 프로젝트의 라이선스 고지이며, 이 확장 자체의 라이선스를 지정한 것은 아닙니다.

이 문서는 Aidoku·BookOrbit 및 직접 사용하는 Aidoku SDK의 관계와 출처를 설명합니다. 그 밖의 Rust 의존성 목록과 버전은 `Cargo.toml`과 `Cargo.lock`에서 관리하며, 각 의존성에는 해당 배포본의 라이선스가 적용됩니다.

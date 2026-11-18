# BookOrbit for Aidoku

[Aidoku](https://github.com/Aidoku/Aidoku)에서 개인 [BookOrbit](https://github.com/bookorbit/bookorbit) 서버의 **CBZ·CBR·CB7 만화**를 탐색하고 읽는 비공식 확장 소스입니다.

- 지원하는 만화 파일이 있는 라이브러리별 탐색과 제목 검색
- 같은 시리즈의 도서를 하나의 작품과 챕터로 표시
- 서버의 표지·도서 정보·페이지 제공

## 설치 및 사용

Aidoku **0.8.3 이상**과 API 키를 지원하는 BookOrbit 서버가 필요합니다. Aidoku의 소스 목록 관리에 다음 URL을 추가하세요.

```text
https://10000je.github.io/aidoku-bookorbit-extension/index.min.json
```

1. 소스 목록을 새로 고치고 **BookOrbit**을 설치합니다.
2. BookOrbit의 **Settings → You → API keys**에서 사용할 라이브러리에 접근 가능한 읽기 전용 API 키를 만듭니다.
3. Aidoku의 소스 설정에 **서버 기본 URL**과 **API 키**를 입력합니다. 서버 URL은 `https://books.example.com`처럼 입력하며 `/api/v1`은 붙이지 않습니다.
4. BookOrbit 소스를 열어 라이브러리를 탐색하거나 제목을 검색한 뒤 작품의 챕터를 선택해 읽습니다.

새 버전은 소스 목록을 새로 고쳐 업데이트할 수 있습니다. AIX 파일은 [GitHub Releases](https://github.com/10000Je/aidoku-bookorbit-extension/releases)에서도 받을 수 있습니다. EPUB·PDF 열람과 BookOrbit 읽기 진도 동기화는 현재 지원하지 않습니다.

[상세 사용 안내](docs/USAGE.md) · [개발·배포](docs/DEVELOPMENT.md) · [오픈소스 프로젝트 및 라이선스 고지](THIRD_PARTY_NOTICES.md)

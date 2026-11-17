#!/usr/bin/env python3
"""사용자가 명시적으로 실행하는 읽기 전용 REST 점검. Python 표준 라이브러리만 사용한다."""

import argparse
import json
import os
import re
import sys
import time
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit, urlunsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener


class ProbeError(Exception):
    pass


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, new_url):
        raise ProbeError("리다이렉트를 거부했습니다. 최종 서버 기본 URL을 입력하세요.")


def normalize_url(value):
    value = value.strip()
    lowered = value.lower()
    if any(ord(char) < 32 or ord(char) == 127 for char in value) or "\\" in value:
        raise ProbeError("서버 URL에 허용되지 않는 문자가 있습니다.")
    try:
        parsed = urlsplit(value)
        _ = parsed.port  # 잘못된 포트는 ValueError를 발생시킨다.
        valid = (parsed.scheme in ("http", "https") and parsed.hostname
                 and parsed.username is None and parsed.password is None
                 and "?" not in value and "#" not in value)
    except ValueError:
        valid = False
    if not valid or any(part in (".", "..") for part in parsed.path.split("/")):
        raise ProbeError("인증 정보·쿼리·API 경로가 없는 서버 기본 URL을 입력하세요.")
    if any(encoded in lowered for encoded in ("%2e", "%2f", "%5c")):
        raise ProbeError("URL 경로에 인코딩된 구분자를 사용할 수 없습니다.")
    path = parsed.path.rstrip("/")
    if path.endswith(("/api/v1", "/opds")):
        raise ProbeError("서버 기본 URL에서 /api/v1 또는 /opds를 제거하세요.")
    return urlunsplit((parsed.scheme, parsed.netloc, path, "", ""))


def positive_id(value):
    if isinstance(value, bool) or not isinstance(value, int) or not 0 < value <= 2147483647:
        raise ProbeError("서버 응답의 도서·파일 ID가 유효하지 않습니다.")
    return value


class Client:
    def __init__(self, base, key):
        self.base = normalize_url(base)
        self.key = key.strip()
        if not self.key or len(self.key) > 512 or not all(33 <= ord(c) <= 126 for c in self.key):
            raise ProbeError("유효한 BOOKORBIT_API_KEY를 로컬 환경 변수로 설정하세요.")
        self.opener = build_opener(NoRedirect())

    def request(self, path, body=None, image=False):
        # 모든 경로는 아래 main에서 검증된 정수 ID 또는 고정 문자열로만 생성한다.
        payload = json.dumps(body).encode() if body is not None else None
        request = Request(self.base + "/api/v1/" + path, data=payload, headers={
            "Authorization": "Bearer " + self.key,
            "Accept": "image/*" if image else "application/json",
            **({"Content-Type": "application/json"} if body is not None else {}),
        })
        for attempt in range(3):
            try:
                with self.opener.open(request, timeout=20) as response:
                    if image:
                        if not response.headers.get_content_type().startswith("image/"):
                            raise ProbeError("페이지 응답이 이미지가 아닙니다.")
                        count = 0
                        while chunk := response.read(65536):
                            count += len(chunk)
                            if count > 64 * 1024 * 1024:
                                raise ProbeError("이미지 점검 크기 한도(64 MiB)를 초과했습니다.")
                        if not count:
                            raise ProbeError("빈 이미지 응답입니다.")
                        return count
                    data = response.read(4 * 1024 * 1024 + 1)
                    if len(data) > 4 * 1024 * 1024:
                        raise ProbeError("JSON 응답이 크기 한도를 초과했습니다.")
                    try:
                        value = json.loads(data)
                    except (ValueError, UnicodeError):
                        raise ProbeError("JSON 응답 형식이 올바르지 않습니다.") from None
                    if not isinstance(value, dict):
                        raise ProbeError("JSON 객체 응답이 필요합니다.")
                    return value
            except HTTPError as error:
                status = error.code
                retry_after = error.headers.get("Retry-After")
                error.close()
                if status == 429 and attempt < 2:
                    delay = 1 << attempt
                    if retry_after is not None:
                        try:
                            specified = int(retry_after)
                        except ValueError:
                            specified = -1
                        if not 0 <= specified <= 10:
                            raise ProbeError("요청 제한(429). 지정된 대기 후 다시 실행하세요.") from None
                        delay = max(delay, specified)
                    time.sleep(delay)
                    continue
                raise ProbeError(f"HTTP {status}. 인증·권한·파일·서버 상태를 확인하세요.") from None
            except (URLError, OSError, TimeoutError):
                raise ProbeError("서버 연결 실패. 주소·네트워크·TLS 인증서를 확인하세요.") from None
        raise ProbeError("요청 제한(429). 나중에 다시 실행하세요.")


def run(client, args):
    info = client.request("app-info")
    version = str(info.get("version", ""))
    print("서버 버전:", version if re.fullmatch(r"v?\d+\.\d+\.\d+[a-zA-Z0-9.+-]{0,25}", version) else "미확인")
    if client.request("auth/me").get("authenticationMethod") != "api_key":
        raise ProbeError("API 키 인증으로 확인되지 않았습니다.")
    print("API 키 인증: 확인")
    body = {
        "pagination": {"page": 0, "size": 40},
        "sort": [{"field": "title", "dir": "asc"}],
        "collapseSeries": False, "collapseLinked": False, "q": args.query,
        "filter": {"type": "group", "join": "AND", "rules": [
            {"type": "rule", "field": "format", "operator": "includesAny", "value": ["cbz", "cbr"]},
            {"type": "rule", "field": "fileAvailability", "operator": "isPresent"},
        ]},
    }
    listing = client.request("books/query", body)
    items = listing.get("items")
    if not isinstance(items, list) or not isinstance(listing.get("total"), int):
        raise ProbeError("목록 응답 계약이 다릅니다.")
    print("검색/목록:", len(items), "개, 전체", listing["total"], "개")
    if listing["total"] > 40:
        body["pagination"]["page"] = 1
        second = client.request("books/query", body)
        if second.get("page") != 1 or not isinstance(second.get("items"), list):
            raise ProbeError("다음 페이지 응답을 확인할 수 없습니다.")
        print("다음 목록 페이지: 확인")
    if args.book_id is None and not items:
        print("빈 라이브러리/검색 결과: 연결 확인. 챕터·이미지 검증은 수행하지 않았습니다.")
        return
    selected_id = args.book_id if args.book_id is not None else items[0].get("id")
    book_id = positive_id(selected_id)
    detail = client.request(f"books/{book_id}")
    if detail.get("id") != book_id or not isinstance(detail.get("files"), list):
        raise ProbeError("도서 상세 응답 계약이 다릅니다.")
    if detail.get("coverSource"):
        client.request(f"books/{book_id}/thumbnail", image=True)
        print("인증 표지: 확인")
    files = [file for file in detail["files"] if isinstance(file, dict)
             and file.get("role") in ("primary", "content")
             and str(file.get("format", "")).lower() in ("cbz", "cbr")]
    if not files:
        raise ProbeError("선택한 도서에 CBZ/CBR 만화 파일이 없습니다.")
    for file in files:
        file_id = positive_id(file.get("id"))
        path = f"cbz/files/{file_id}/pages"
        count = client.request(path).get("pageCount")
        if isinstance(count, bool) or not isinstance(count, int) or not 0 < count <= 20000:
            raise ProbeError("페이지 수가 없거나 유효 범위를 벗어났습니다.")
        indices = range(count) if args.all_pages else sorted({0, count - 1})
        for index in indices:
            client.request(f"{path}/{index}", image=True)
        print(f"파일 {file_id}: {count}페이지, 인증 이미지 {len(indices)}개 확인")
    print("REST 점검 완료. Aidoku Wasm 실행·이미지 디코딩·실기기 검증은 별도로 필요합니다.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--query", default="", help="검증할 제목 검색어")
    parser.add_argument("--book-id", type=int, help="목록 첫 도서 대신 검사할 BookOrbit 도서 ID")
    parser.add_argument("--all-pages", action="store_true", help="첫/마지막 페이지 대신 모든 페이지 순차 조회")
    args = parser.parse_args()
    base, key = os.environ.get("BOOKORBIT_URL"), os.environ.get("BOOKORBIT_API_KEY")
    if not base or not key:
        print("SKIP: BOOKORBIT_URL과 BOOKORBIT_API_KEY를 로컬 환경 변수로 설정하세요.")
        return 2
    try:
        if len(args.query.encode("utf-16-le")) // 2 > 200:
            raise ProbeError("검색어는 UTF-16 기준 200자 이내로 입력하세요.")
        run(Client(base, key), args)
    except ProbeError as error:
        print("실패:", error, file=sys.stderr)
        return 1
    except (ValueError, TypeError, KeyError, AttributeError):
        print("실패: 서버 응답 형식이 계약과 다릅니다.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

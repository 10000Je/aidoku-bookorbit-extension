"""실제 서버 없이 probe의 인증·리다이렉트·스트리밍·재시도를 검증한다."""
import importlib.util
import io
import json
import os
from email.message import Message
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch
from urllib.error import HTTPError

spec = importlib.util.spec_from_file_location(
    "probe", Path(__file__).resolve().parents[1] / "scripts/probe_bookorbit.py"
)
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class Response(io.BytesIO):
    def __init__(self, body, content_type="application/json"):
        super().__init__(body)
        self.headers = Message()
        self.headers["Content-Type"] = content_type


class ProbeTests(unittest.TestCase):
    def client(self):
        return probe.Client("https://books.example.com/shelf/", "bok_synthetic_only")

    def test_normalizes_proxy_path_and_rejects_unsafe_urls(self):
        self.assertEqual(probe.normalize_url(" https://books.example.com/shelf/// "),
                         "https://books.example.com/shelf")
        for value in ("ftp://books.example.com", "https://user:pass@books.example.com",
                      "https://books.example.com/api/v1", "https://books.example.com/a/../b",
                      "https://books.example.com:99999", "https://books.example.com?token=x"):
            with self.subTest(value=value), self.assertRaises(probe.ProbeError):
                probe.normalize_url(value)

    def test_refuses_redirects_instead_of_forwarding_credentials(self):
        with self.assertRaises(probe.ProbeError):
            probe.NoRedirect().redirect_request(None, None, 302, "", {}, "https://evil.example")

    def test_authentication_is_in_one_header_and_search_is_post(self):
        client = self.client()
        client.opener = Mock()
        client.opener.open.return_value = Response(b'{"total":0}')
        self.assertEqual(client.request("books/query", {"q":"title"}), {"total":0})
        request = client.opener.open.call_args.args[0]
        self.assertEqual(request.full_url, "https://books.example.com/shelf/api/v1/books/query")
        self.assertEqual(request.get_method(), "POST")
        self.assertEqual(request.get_header("Authorization"), "Bearer bok_synthetic_only")
        self.assertNotIn("bok_synthetic_only", request.full_url)
        self.assertEqual(client.opener.open.call_args.kwargs["timeout"], 20)

    def test_rate_limit_backoff_and_recovery_are_bounded(self):
        client = self.client()
        client.opener = Mock()
        client.opener.open.side_effect = [
            HTTPError("https://books.example.com",429,"",{},None),
            HTTPError("https://books.example.com",429,"",{"Retry-After":"3"},None),
            Response(b'{}'),
        ]
        with patch.object(probe.time, "sleep") as sleep:
            self.assertEqual(client.request("auth/me"), {})
        self.assertEqual([call.args[0] for call in sleep.call_args_list], [1,3])
        self.assertEqual(client.opener.open.call_count, 3)

    def test_long_retry_after_returns_error_without_sleeping(self):
        client = self.client()
        client.opener = Mock()
        client.opener.open.side_effect = HTTPError("",429,"",{"Retry-After":"60"},None)
        with patch.object(probe.time,"sleep") as sleep, self.assertRaises(probe.ProbeError):
            client.request("auth/me")
        sleep.assert_not_called()

    def test_image_stream_requires_image_content_and_nonempty_bytes(self):
        client = self.client()
        client.opener = Mock()
        client.opener.open.return_value = Response(b"synthetic-image", "image/png")
        self.assertEqual(client.request("cbz/files/501/pages/0", image=True), 15)
        for body, kind in [(b"", "image/png"), (b"login", "text/html")]:
            client.opener.open.return_value = Response(body,kind)
            with self.assertRaises(probe.ProbeError):
                client.request("cbz/files/501/pages/0", image=True)

    def test_malformed_response_and_auth_errors_do_not_echo_secrets(self):
        client = self.client()
        client.opener = Mock()
        client.opener.open.return_value = Response(b"bok_synthetic_only")
        with self.assertRaises(probe.ProbeError) as context:
            client.request("auth/me")
        self.assertNotIn("bok_synthetic_only", str(context.exception))
        client.opener.open.side_effect = HTTPError("",401,"bok_synthetic_only",{},None)
        with self.assertRaises(probe.ProbeError) as context:
            client.request("auth/me")
        self.assertNotIn("bok_synthetic_only", str(context.exception))

    def test_missing_credentials_skip_without_network(self):
        with patch.dict(os.environ, {}, clear=True), patch.object(probe.sys, "argv", ["probe"]), \
             patch.object(probe, "build_opener") as opener, patch("sys.stdout", new_callable=io.StringIO):
            self.assertEqual(probe.main(), 2)
        opener.assert_not_called()

    def test_cb7_probe_checks_search_filter_and_first_last_images(self):
        detail = json.loads((Path(__file__).parent / "fixtures/cb7-book-detail.json").read_text())
        client = Mock()
        client.request.side_effect = [
            {"version": "3.8.0"},
            {"authenticationMethod": "api_key"},
            {"items": [{"id": 142}], "total": 1},
            detail,
            {"pageCount": 3}, 15, 15,
            {"pageCount": 2}, 15, 15,
        ]
        args = SimpleNamespace(query="", book_id=None, all_pages=False)
        with patch("sys.stdout", new_callable=io.StringIO):
            probe.run(client, args)
        calls = client.request.call_args_list
        self.assertEqual(calls[2].args[1]["filter"]["rules"][0]["value"], ["cbz", "cbr", "cb7"])
        self.assertEqual([call.args[0] for call in calls[4:]], [
            "cbz/files/702/pages", "cbz/files/702/pages/0", "cbz/files/702/pages/2",
            "cbz/files/701/pages", "cbz/files/701/pages/0", "cbz/files/701/pages/1",
        ])
        self.assertEqual([call.kwargs.get("image", False) for call in calls[4:]],
                         [False, True, True, False, True, True])


if __name__ == "__main__":
    unittest.main()

#!/usr/bin/env python3
"""Copy every linked file from a static test server into a browser container."""

import os
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urljoin, urlsplit
from urllib.request import urlopen


BASE_URL = os.environ.get("XEZIM_STAGE_BASE_URL", "http://xezim-luna04-builder:4180/")
DESTINATION = Path(os.environ.get("XEZIM_STAGE_DIRECTORY", "/tmp/luna06-local"))
base = urlsplit(BASE_URL)
origin = f"{base.scheme}://{base.netloc}"


class Links(HTMLParser):
    def __init__(self):
        super().__init__()
        self.hrefs = []
        self.title = ""

    def handle_starttag(self, tag, attrs):
        values = dict(attrs)
        link = values.get("href") if tag in {"a", "link"} else values.get("src") if tag == "script" else None
        if link:
            self.hrefs.append(link)

    def handle_data(self, data):
        if data.strip().startswith("Directory listing for "):
            self.title = data.strip()


queue = [BASE_URL, *(urljoin(BASE_URL, name) for name in ("assets/", "wasm/", "fixtures/", "surfer/"))]
visited = set()
while queue:
    url = queue.pop(0)
    if url in visited:
        continue
    visited.add(url)
    with urlopen(url) as response:
        body = response.read()
        content_type = response.headers.get_content_type()
        final_url = response.geturl()
    parsed = urlsplit(final_url)
    if f"{parsed.scheme}://{parsed.netloc}" != origin:
        continue
    relative = parsed.path.lstrip("/")
    if not relative or relative.endswith("/"):
        parser = Links()
        if content_type == "text/html":
            parser.feed(body.decode("utf-8", errors="replace"))
        if parser.title.startswith("Directory listing for "):
            for href in parser.hrefs:
                child = urljoin(final_url, href)
                child_url = urlsplit(child)
                if f"{child_url.scheme}://{child_url.netloc}" == origin and child_url.path not in {"/../", "/./"}:
                    queue.append(child)
            continue
        relative = f"{relative}index.html"
    target = DESTINATION / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(body)
    if content_type == "text/html":
        parser = Links()
        parser.feed(body.decode("utf-8", errors="replace"))
        for href in parser.hrefs:
            child = urljoin(final_url, href)
            child_url = urlsplit(child)
            if f"{child_url.scheme}://{child_url.netloc}" == origin:
                queue.append(child)

print(f"Staged {len(visited)} URLs from {BASE_URL} into {DESTINATION}")

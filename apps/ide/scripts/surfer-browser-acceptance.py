#!/usr/bin/env python3
"""Real-browser acceptance for the pinned Surfer iframe adapter."""

import argparse
import json
import os
import time
from pathlib import Path

from camoufox.sync_api import Camoufox
from playwright.sync_api import expect


URL = os.environ.get("XEZIM_SURFER_TEST_URL", "http://127.0.0.1:4216/")


def wait_for_log(page, messages, fragment, timeout=30):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if any(fragment in entry["text"] for entry in messages):
            return
        page.wait_for_timeout(100)
    raise AssertionError(f"Timed out waiting for Surfer log {fragment!r}; logs={messages!r}")


def ensure_app_loaded(page):
    # The provisioned Camoufox static harness does not execute this app's entry
    # module on navigation. The normal page entry is still a separate release gate.
    if page.locator("#project-select").count() == 0:
        page.evaluate("import(document.scripts[0].src)")
    expect(page.locator("#project-select")).to_be_visible(timeout=60_000)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, default=Path("surfer-app-artifacts"))
    parser.add_argument("--headed", action="store_true", help="use Xvfb/WebGL in the browser container")
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)

    console = []
    errors = []
    with Camoufox(headless=not args.headed) as browser:
        page = browser.new_page(viewport={"width": 1440, "height": 1000}, device_scale_factor=1)
        page.on("console", lambda message: console.append({"type": message.type, "text": message.text, "url": message.location.get("url")}))
        page.on("pageerror", lambda error: errors.append(str(error)))
        page.on("requestfailed", lambda request: errors.append(f"request failed: {request.url}: {request.failure}"))
        response = page.goto(URL, wait_until="domcontentloaded", timeout=60_000)
        assert response and response.status == 200, f"IDE HTTP response was {response.status if response else 'missing'}"
        ensure_app_loaded(page)

        page.get_by_role("tab", name="WAVEFORMS").click()
        iframe = page.locator("#surferFrame")
        expect(iframe).to_be_visible()
        try:
            expect(page.locator(".surfer-readiness.ready")).to_have_text("Surfer v0.7.0 ready", timeout=120_000)
        except AssertionError:
            print(f"DEBUG body={page.locator('body').inner_text()!r}")
            print(f"DEBUG frames={[frame.url for frame in page.frames]!r}")
            print(f"DEBUG console={console[-40:]!r} errors={errors!r}")
            raise
        viewer_origin = iframe.evaluate("frame => frame.contentWindow.location.origin")
        assert viewer_origin == page.evaluate("location.origin"), f"Surfer is not same-origin: {viewer_origin}"
        frame = page.frame_locator("#surferFrame")
        expect(frame.locator("canvas")).to_have_count(1, timeout=60_000)
        iframe.screenshot(path=str(args.output_dir / "01-surfer-ready.png"))

        page.get_by_role("button", name="Load sample VCD").click()
        wait_for_log(page, console, "WaveBodyLoaded")
        iframe.screenshot(path=str(args.output_dir / "02-vcd-loaded.png"))

        page.get_by_role("button", name="Add top.count").click()
        wait_for_log(page, console, "Loaded 1 variables")
        page.get_by_role("button", name="Set cursor 25 ns").click()
        wait_for_log(page, console, "CursorSet(25)")
        iframe.screenshot(path=str(args.output_dir / "03-signal-and-cursor.png"))

        before_zoom = iframe.screenshot(path=str(args.output_dir / "03-signal-and-cursor.png"))
        page.get_by_role("button", name="Zoom 0–40 ns").click()
        page.wait_for_timeout(1200)
        after_zoom = iframe.screenshot(path=str(args.output_dir / "04-zoomed.png"))
        if before_zoom == after_zoom:
            print(f"DEBUG zoom logs={console[-50:]!r}")
            print(f"DEBUG viewer={iframe.evaluate('frame => ({body:frame.contentDocument.body.innerText.slice(0,1200), ready:frame.contentWindow.__xezim_surfer_ready})')!r}")
        assert before_zoom != after_zoom, "ZoomToRange did not change the rendered Surfer iframe screenshot"

        # Reload the actual iframe and confirm the nonce handshake makes the
        # adapter ready again before it sends another command.
        generation_before_reload = int(page.locator(".surfer-readiness [data-ready-generation]").get_attribute("data-ready-generation"))
        page.evaluate("document.querySelector('#surferFrame').contentWindow.location.reload()")
        page.wait_for_function(
            "(generation) => Number(document.querySelector('.surfer-readiness [data-ready-generation]')?.dataset.readyGeneration) > generation",
            arg=generation_before_reload,
            timeout=120_000,
        )
        expect(page.locator(".surfer-readiness.ready")).to_contain_text("Surfer v0.7.0 ready")
        page.get_by_role("button", name="Load sample VCD").click()
        wait_for_log(page, console, "WaveBodyLoaded", timeout=30)
        page.get_by_role("button", name="Add top.count").click()
        wait_for_log(page, console, "Loaded 1 variables")
        page.get_by_role("button", name="Set cursor 25 ns").click()
        wait_for_log(page, console, "CursorSet(25)")
        iframe.screenshot(path=str(args.output_dir / "05-reloaded-and-controlled.png"))

        console_errors = [entry for entry in console if entry["type"] == "error"]
        assert not errors, f"Browser page errors or failed requests: {errors}"
        assert not console_errors, f"Browser console errors: {console_errors}"
        report = {
            "result": "passed",
            "browser_version": browser.version,
            "url": URL,
            "same_origin": True,
            "verified": ["same-origin VCD load", "AddVariables top.count", "CursorSet 25 ns", "ZoomToRange 0-40 ns", "iframe reload readiness handshake"],
            "screenshot_bytes_before_zoom": len(before_zoom),
            "screenshot_bytes_after_zoom": len(after_zoom),
            "console_errors": console_errors,
            "page_errors": errors,
            "console": console,
            "screenshots": ["01-surfer-ready.png", "02-vcd-loaded.png", "03-signal-and-cursor.png", "04-zoomed.png", "05-reloaded-and-controlled.png"],
        }
        (args.output_dir / "browser-test-log.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(json.dumps({key: value for key, value in report.items() if key != "console"}, indent=2))


if __name__ == "__main__":
    main()

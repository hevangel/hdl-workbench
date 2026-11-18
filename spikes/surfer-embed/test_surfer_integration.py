#!/usr/bin/env python3
"""Browser smoke for the pinned Surfer bundle; run inside the Playwright/Camoufox image."""

import argparse
import json
from pathlib import Path

from camoufox.sync_api import Camoufox


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--url", default="http://127.0.0.1:4173/")
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts"))
    parser.add_argument("--headed", action="store_true", help="use the browser container's Xvfb display for WebGL")
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)

    events = []
    errors = []
    with Camoufox(headless=not args.headed) as browser:
        page = browser.new_page(viewport={"width": 1440, "height": 1000}, device_scale_factor=1)
        page.on("console", lambda message: events.append({"kind": "console", "type": message.type, "text": message.text}))
        page.on("pageerror", lambda error: errors.append(str(error)))
        page.on("requestfailed", lambda request: events.append({"kind": "requestfailed", "url": request.url, "error": request.failure}))
        page.on("response", lambda response: events.append({"kind": "response", "status": response.status, "url": response.url}) if "/assets/surfer/" in response.url or "/fixture-counter.vcd" in response.url else None)

        response = page.goto(args.url, wait_until="domcontentloaded", timeout=60000)
        assert response and response.status == 200, f"Harness HTTP status was {response.status if response else 'missing'}"
        page.wait_for_function("document.querySelector('#status')?.dataset.ready === 'true'", timeout=120000)
        frame = page.locator("#surferFrame")

        def capture(name: str) -> None:
            frame.screenshot(path=str(args.output_dir / name))

        capture("01-viewer-ready.png")
        page.get_by_role("button", name="Load fixture", exact=True).click()
        page.wait_for_timeout(2500)
        capture("02-fixture-loaded.png")
        page.get_by_role("button", name="Add top.count").click()
        page.wait_for_timeout(1500)
        capture("03-signal-added.png")
        page.get_by_role("button", name="Set cursor to 25 ns").click()
        page.wait_for_timeout(1000)
        capture("04-cursor-set.png")

        viewer_state = page.locator("#surferFrame").evaluate("frame => ({url: frame.contentWindow.location.href, title: frame.contentDocument.title, canvasCount: frame.contentDocument.querySelectorAll('canvas').length, bodyText: frame.contentDocument.body.innerText.slice(0, 1200)})")
        log_text = page.locator("#log").inner_text()
        wasm_logs = [event["text"] for event in events if event.get("kind") == "console" and event.get("type") == "log"]
        console_errors = [event["text"] for event in events if event.get("kind") == "console" and event.get("type") == "error"]
        assert viewer_state["canvasCount"] > 0, f"Surfer canvas was not rendered: {viewer_state}"
        assert "Sorry, Surfer crashed" not in viewer_state["bodyText"], f"Viewer crashed: {viewer_state['bodyText']}"
        assert log_text.count(" sent ") == 3, f"Expected load/add/cursor wrapper messages: {log_text}"
        assert "fixture-counter.vcd" in log_text and "AddVariables" in log_text and "CursorSet" in log_text, log_text
        assert any("WaveBodyLoaded" in message or "Loaded the body" in message for message in wasm_logs), wasm_logs
        assert any("Loaded 1 variables" in message for message in wasm_logs), wasm_logs
        assert any("CursorSet(25)" in message for message in wasm_logs), wasm_logs
        assert not errors, f"Browser page errors: {errors}"
        assert not console_errors, f"Console errors: {console_errors}"

        before_blob_log_count = len(wasm_logs)
        page.get_by_role("button", name="Load fixture Blob URL").click()
        page.wait_for_timeout(2500)
        frame.screenshot(path=str(args.output_dir / "05-blob-attempt.png"))
        all_wasm_logs = [event["text"] for event in events if event.get("kind") == "console" and event.get("type") == "log"]
        blob_logs = all_wasm_logs[before_blob_log_count:]
        blob_loaded = any("WaveBodyLoaded" in message and "blob:" in message for message in blob_logs) or any("Loaded the body of blob:" in message for message in blob_logs)
        if blob_loaded:
            page.get_by_role("button", name="Add top.count").click()
            page.wait_for_timeout(1500)
            page.get_by_role("button", name="Set cursor to 25 ns").click()
            page.wait_for_timeout(1000)
            frame.screenshot(path=str(args.output_dir / "06-blob-signal-cursor.png"))
            all_wasm_logs = [event["text"] for event in events if event.get("kind") == "console" and event.get("type") == "log"]
            blob_logs = all_wasm_logs[before_blob_log_count:]
        final_log_text = page.locator("#log").inner_text()
        all_console_errors = [event["text"] for event in events if event.get("kind") == "console" and event.get("type") == "error"]
        assert not errors, f"Browser page errors after Blob test: {errors}"
        assert not all_console_errors, f"Console errors after Blob test: {all_console_errors}"
        report = {
            "browser": "Firefox/Camoufox",
            "browser_version": browser.version,
            "base_url": args.url,
            "viewer_state": viewer_state,
            "harness_events": final_log_text.splitlines(),
            "browser_errors": errors,
            "console_errors": all_console_errors,
            "browser_events": events,
            "wasm_logs": wasm_logs,
            "blob_attempt": {"loaded": blob_loaded, "wasm_logs": blob_logs},
            "screenshots": ["01-viewer-ready.png", "02-fixture-loaded.png", "03-signal-added.png", "04-cursor-set.png", "05-blob-attempt.png"] + (["06-blob-signal-cursor.png"] if blob_loaded else []),
            "harness_status": page.locator("#status").inner_text(),
            "harness_log": final_log_text,
        }
        (args.output_dir / "browser-test-log.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
        assert blob_loaded, f"Pinned Surfer did not load the Blob URL: {blob_logs}"
        assert any("Loaded 1 variables" in message for message in blob_logs), blob_logs
        assert any("CursorSet(25)" in message for message in blob_logs), blob_logs
        print(json.dumps({"result": "passed", "browser_version": browser.version, "viewer_state": viewer_state, "verified_messages": ["LoadUrl", "AddVariables", "CursorSet(25)"], "blob_load": blob_loaded, "blob_control": {"signal_added": any("Loaded 1 variables" in message for message in blob_logs), "cursor_set": any("CursorSet(25)" in message for message in blob_logs)}, "screenshots": report["screenshots"], "browser_errors": errors, "console_errors": all_console_errors}, indent=2))


if __name__ == "__main__":
    main()

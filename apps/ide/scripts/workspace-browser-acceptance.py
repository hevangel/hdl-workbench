#!/usr/bin/env python3
"""Firefox acceptance test for persistent workspace behavior.

Run this inside the provisioned Camoufox/Playwright image after serving the
built IDE on a same-origin localhost URL. Requires Python Camoufox and the
Playwright assertions bundled with that image.
"""

import io
import os
import tempfile
import zipfile

from camoufox.sync_api import Camoufox
from playwright.sync_api import expect


URL = os.environ.get("XEZIM_WORKSPACE_TEST_URL", "http://127.0.0.1:4195/")
PERSISTED_MARKER = "LUNA06_RELOAD_PERSISTED"
LOCAL_DRAFT_MARKER = "LUNA06_LOCAL_DRAFT"
REMOTE_MARKER = "LUNA06_REMOTE_VERSION"
REPLACEMENT_MARKER = "LUNA06_EXPLICIT_REPLACEMENT"
entry_import_workarounds = 0


def ensure_app_loaded(page):
    """Camoufox test-container workaround; production code never uses this path."""
    global entry_import_workarounds
    if page.locator("#project-select").count() == 0:
        entry_import_workarounds += 1
        page.evaluate("import(document.scripts[0].src)")
    try:
        expect(page.locator("#project-select")).to_be_visible(timeout=60_000)
    except AssertionError:
        print(f"DEBUG startup body={page.locator('body').inner_text()!r} root={page.locator('#root').inner_html()[:800]!r}")
        raise


def navigate_to_app(page, url=URL):
    page.goto(url, wait_until="domcontentloaded", timeout=60_000)
    ensure_app_loaded(page)


def reload_app(page):
    page.reload(wait_until="domcontentloaded", timeout=60_000)
    ensure_app_loaded(page)


def append_line(page, marker):
    editor = page.locator(".monaco-editor").first
    editor.click()
    page.keyboard.press("Control+End")
    page.keyboard.press("Enter")
    page.keyboard.insert_text(f"// {marker}")


def open_counter(page):
    locator = page.locator('button[aria-label="Open rtl/counter.sv"]')
    locator.wait_for(state="visible", timeout=60_000)
    locator.click()
    expect(page.locator(".monaco-editor")).to_be_visible()


def wait_saved(page):
    page.get_by_role("button", name="Save", exact=True).click()
    expect(page.locator('.workspace-notice[role="status"]')).to_contain_text("All project files are saved", timeout=10_000)


def defer_autosave_for_conflict(page):
    # Keep the local draft dirty long enough to exercise the competing write.
    page.evaluate("""() => {
      const original = window.setTimeout.bind(window);
      window.setTimeout = (callback, delay, ...args) => original(callback, delay === 450 ? 5000 : delay, ...args);
    }""")


def main():
    trusted_host = os.environ.get("XEZIM_TRUSTED_HOST")
    prefs = {"dom.securecontext.allowlist": trusted_host} if trusted_host else {}
    with Camoufox(headless=False, firefox_user_prefs=prefs) as browser:
        print(f"Browser version: {browser.version}")
        context = browser.new_context(viewport={"width": 1440, "height": 1000}, device_scale_factor=1)
        page = context.new_page()
        page_errors = []
        page.on("pageerror", lambda error: page_errors.append(str(error)))
        page.on("console", lambda message: page_errors.append(f"console {message.type}: {message.text}") if message.type in {"error", "warning"} else None)
        page.on("requestfailed", lambda request: page_errors.append(f"request failed {request.url}: {request.failure}"))
        navigate_to_app(page)
        expect(page.locator("#project-select")).to_have_value("counter-lab")
        expect(page.locator("#authority-mode")).to_have_value("browser-only")
        open_counter(page)

        # A debounced save and explicit receipt survive a full page reload.
        append_line(page, PERSISTED_MARKER)
        wait_saved(page)
        reload_app(page)
        open_counter(page)
        expect(page.locator(".view-lines")).to_contain_text(PERSISTED_MARKER, timeout=10_000)

        # A second tab writing the same path leaves the first draft intact and
        # requires the user to choose reload or confirmed replacement.
        second = context.new_page()
        navigate_to_app(second)
        open_counter(second)
        defer_autosave_for_conflict(page)
        append_line(page, LOCAL_DRAFT_MARKER)
        append_line(second, REMOTE_MARKER)
        wait_saved(second)
        conflict = page.locator('.workspace-notice[role="alert"]')
        expect(conflict).to_contain_text("Storage conflict", timeout=10_000)
        expect(page.locator(".view-lines")).to_contain_text(LOCAL_DRAFT_MARKER, timeout=5_000)
        page.get_by_role("button", name="Reload stored version").click()
        expect(page.locator(".view-lines")).to_contain_text(REMOTE_MARKER, timeout=5_000)
        assert LOCAL_DRAFT_MARKER not in page.locator(".view-lines").inner_text(), "reload must replace the draft with the stored version"

        # Replacing a cross-tab version requires explicit confirmation.
        defer_autosave_for_conflict(page)
        append_line(page, REPLACEMENT_MARKER)
        append_line(second, "LUNA06_REMOTE_AFTER_RELOAD")
        wait_saved(second)
        expect(conflict).to_contain_text("Storage conflict", timeout=10_000)
        page.once("dialog", lambda dialog: dialog.accept())
        page.get_by_role("button", name="Replace stored version…").click()
        try:
            expect(page.locator('.workspace-notice[role="status"]')).to_contain_text("Stored your draft", timeout=10_000)
        except AssertionError:
            alert_text = page.locator('.workspace-notice[role="alert"]').all_inner_texts()
            status_text = page.locator('.workspace-notice[role="status"]').all_inner_texts()
            editor_text = page.locator('.view-lines').inner_text()
            print(f"DEBUG replace alert={alert_text!r} status={status_text!r} editor={editor_text!r} errors={page_errors!r}")
            raise
        reload_app(second)
        open_counter(second)
        expect(second.locator(".view-lines")).to_contain_text(REPLACEMENT_MARKER, timeout=10_000)

        # Export includes editor content; import a DEFLATE ZIP with a data
        # descriptor and ensure the selected project survives another reload.
        with second.expect_download() as download_info:
            second.get_by_role("button", name="Export ZIP").click()
        download = download_info.value
        with tempfile.TemporaryDirectory() as temp_dir:
            exported_path = os.path.join(temp_dir, "counter-lab.zip")
            download.save_as(exported_path)
            with zipfile.ZipFile(exported_path) as exported:
                assert "rtl/counter.sv" in exported.namelist()
                source = exported.read("rtl/counter.sv")
                assert REPLACEMENT_MARKER.encode() in source

            class Unseekable(io.BytesIO):
                def seekable(self):
                    return False

                def seek(self, *_args, **_kwargs):
                    raise io.UnsupportedOperation("unseekable test archive")

            compressed = Unseekable()
            with zipfile.ZipFile(compressed, "w", compression=zipfile.ZIP_DEFLATED) as archive:
                with zipfile.ZipFile(exported_path) as exported:
                    for name in exported.namelist():
                        archive.writestr(name, exported.read(name))
            archive_bytes = compressed.getvalue()
            assert archive_bytes[6] & 0x08, "fixture archive must exercise ZIP data descriptors"
            second.locator('input[type="file"][accept*=".zip"]').set_input_files({
                "name": "counter-lab-deflated.zip",
                "mimeType": "application/zip",
                "buffer": archive_bytes,
            })

        selected = second.locator("#project-select")
        expect(selected).not_to_have_value("counter-lab", timeout=10_000)
        imported_id = selected.input_value()
        reload_app(second)
        expect(second.locator("#project-select")).to_have_value(imported_id)
        open_counter(second)
        expect(second.locator(".view-lines")).to_contain_text(REPLACEMENT_MARKER, timeout=10_000)

        # Rename and delete operate on project-relative paths and update the
        # visible tree/tabs after durable writes.
        second.once("dialog", lambda dialog: dialog.accept("rtl/counter-renamed.sv"))
        second.get_by_role("button", name="File actions", exact=True).click()
        second.get_by_role("button", name="Rename active file…").click()
        expect(second.get_by_role("button", name="Open rtl/counter-renamed.sv")).to_be_visible()
        second.once("dialog", lambda dialog: dialog.accept())
        second.get_by_role("button", name="File actions", exact=True).click()
        second.get_by_role("button", name="Delete active file…").click()
        expect(second.get_by_role("button", name="Open rtl/counter-renamed.sv")).to_have_count(0)

        assert not page_errors, f"Unexpected browser errors: {page_errors}"
        print(f"PASS: reload, save receipt, cross-tab conflict, explicit replace, ZIP export/import, project selection, rename, and delete (Camoufox entry-import workarounds: {entry_import_workarounds})")


if __name__ == "__main__":
    main()

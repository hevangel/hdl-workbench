"""#426: a shared build directory supersedes cached runner parameters."""

import os

import pytest

from test_xezim_runner import build, get_runner, run


def test_rebuild_through_another_runner(tmp_path):
    r1 = get_runner("xezim")
    build(r1, tmp_path, parameters={"W": 8})
    run(r1, tmp_path, {"w": 8})
    r2 = get_runner("xezim")
    build(r2, tmp_path, parameters={"W": 5})
    run(r2, tmp_path, {"w": 5})
    run(r1, tmp_path, {"w": 5})


def test_external_rebuild_replaces_an_explicit_override(tmp_path):
    r1 = get_runner("xezim")
    build(r1, tmp_path, parameters={"W": 8})
    run(r1, tmp_path, {"w": 3}, parameters={"W": 3})
    build(get_runner("xezim"), tmp_path, parameters={"W": 5})
    run(r1, tmp_path, {"w": 5})


def test_identical_external_rebuild_replaces_an_override(tmp_path):
    r1 = get_runner("xezim")
    build(r1, tmp_path, parameters={"W": 8})
    run(r1, tmp_path, {"w": 3}, parameters={"W": 3})
    build(get_runner("xezim"), tmp_path, parameters={"W": 8})
    run(r1, tmp_path, {"w": 8})


def test_changed_record_with_preserved_timestamp(tmp_path):
    r1 = get_runner("xezim")
    build(r1, tmp_path, parameters={"W": 8})
    run(r1, tmp_path, {"w": 8})
    record = tmp_path / "build" / "xezim_build.json"
    previous = record.stat()
    build(get_runner("xezim"), tmp_path, parameters={"W": 5})
    os.utime(record, ns=(previous.st_atime_ns, previous.st_mtime_ns))
    run(r1, tmp_path, {"w": 5})


def test_external_rebuild_with_changed_defines(tmp_path):
    r1 = get_runner("xezim")
    build(r1, tmp_path, parameters={"W": 8})
    run(r1, tmp_path, {"w": 3}, parameters={"W": 3})
    build(get_runner("xezim"), tmp_path, parameters={"W": 8}, defines={"FOO": None})
    run(r1, tmp_path, {"w": 8, "foo": 1})


@pytest.mark.parametrize("override, expected", [({"W": 6}, 6), ({}, 1)])
def test_override_after_external_rebuild_is_retained(tmp_path, override, expected):
    r1 = get_runner("xezim")
    build(r1, tmp_path, parameters={"W": 8})
    run(r1, tmp_path, {"w": 8})
    build(get_runner("xezim"), tmp_path, parameters={"W": 5})
    run(r1, tmp_path, {"w": expected}, parameters=override)
    run(r1, tmp_path, {"w": expected})

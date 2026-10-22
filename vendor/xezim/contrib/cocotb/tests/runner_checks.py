"""cocotb side of test_xezim_runner.py: compares runner_dut's outputs with
the values the EXPECT environment variable names."""

import json
import os

import cocotb
from cocotb.triggers import Timer


@cocotb.test()
async def check(dut):
    await Timer(1, "ns")
    expect = json.loads(os.environ["EXPECT"])
    got = {name: int(getattr(dut, name).value) for name in expect}
    assert got == expect

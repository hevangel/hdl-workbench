Training designs for `scripts/build-pgo.sh` when it is run without a
training command. Together with the `tests/perf` shape designs and the
`xezim-bench` workloads they exercise the paths a profile-guided build
must see: the two-state executors (narrow, control-flow and wide), the
settle loop, edge dispatch and skipping, the interpreter, the NBA queue,
loop-heavy clocked blocks and the x-bail backoff. Small on purpose — each
runs in well under a second — so the profile is cheap to regenerate.

//! Sampling thread behind `--profile`.
//!
//! The simulation thread publishes the construct it is executing with one
//! relaxed atomic store (a comb entry index or an edge-block index, tagged
//! in the high word); this thread reads the slot every few dozen
//! microseconds and tallies what it saw. Timing therefore costs the
//! simulation thread a single store per evaluation, and the report converts
//! samples to time from the wall clock the sampler ran for.

use crate::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::JoinHandle;

/// Nothing attributable is running (scheduling, NBA apply, processes).
pub const KIND_OTHER: u64 = 0;
/// A comb entry: `KIND_COMB | entry index`.
pub const KIND_COMB: u64 = 1 << 32;
/// An edge block: `KIND_EDGE | block index`.
pub const KIND_EDGE: u64 = 2 << 32;
/// A compiled process FSM: `KIND_PROC | pid`.
pub const KIND_PROC: u64 = 3 << 32;

/// Interval between samples. The kernel's timer slack stretches this to
/// roughly 60–100 µs, so a run collects about 10k samples a second.
const INTERVAL: Duration = Duration::from_micros(30);

/// What the sampler saw, converted at `finish`.
pub struct SampleTally {
    /// Samples per comb entry index.
    pub comb: Vec<u64>,
    /// Samples per edge-block index.
    pub edge: Vec<u64>,
    /// Samples per compiled process FSM (by pid).
    pub proc: Vec<u64>,
    /// Samples that fell outside comb entries and edge blocks.
    pub other: u64,
    /// Every sample taken.
    pub total: u64,
    /// Wall time the sampler ran for.
    pub elapsed_ns: u64,
    /// Nanoseconds one sample stands for: `elapsed_ns / total`.
    pub ns_per_sample: f64,
}

pub struct ProfSampler {
    /// Slot the simulation thread publishes into.
    pub cur: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<(Vec<u64>, Vec<u64>, Vec<u64>, u64, u64)>>,
    started: Instant,
}

fn bump(v: &mut Vec<u64>, idx: usize) {
    if idx >= v.len() {
        v.resize(idx + 1, 0);
    }
    v[idx] += 1;
}

impl ProfSampler {
    pub fn start() -> Self {
        let cur = Arc::new(AtomicU64::new(KIND_OTHER));
        let stop = Arc::new(AtomicBool::new(false));
        let (cur_t, stop_t) = (cur.clone(), stop.clone());
        let handle = std::thread::Builder::new()
            .name("xezim-prof".into())
            .spawn(move || {
                let (mut comb, mut edge, mut proc) = (Vec::new(), Vec::new(), Vec::new());
                let (mut other, mut total) = (0u64, 0u64);
                while !stop_t.load(Ordering::Acquire) {
                    let v = cur_t.load(Ordering::Relaxed);
                    let idx = (v & 0xffff_ffff) as usize;
                    match v >> 32 {
                        1 => bump(&mut comb, idx),
                        2 => bump(&mut edge, idx),
                        3 => bump(&mut proc, idx),
                        _ => other += 1,
                    }
                    total += 1;
                    std::thread::sleep(INTERVAL);
                }
                (comb, edge, proc, other, total)
            })
            .ok();
        Self {
            cur,
            stop,
            handle,
            started: Instant::now(),
        }
    }

    /// Stop sampling and return the tally.
    pub fn finish(mut self) -> SampleTally {
        self.stop.store(true, Ordering::Release);
        let elapsed_ns = self.started.elapsed().as_nanos() as u64;
        let (comb, edge, proc, other, total) = match self.handle.take().map(|h| h.join()) {
            Some(Ok(t)) => t,
            _ => (Vec::new(), Vec::new(), Vec::new(), 0, 0),
        };
        let ns_per_sample = elapsed_ns as f64 / total.max(1) as f64;
        SampleTally {
            comb,
            edge,
            proc,
            other,
            total,
            elapsed_ns,
            ns_per_sample,
        }
    }
}

impl Drop for ProfSampler {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

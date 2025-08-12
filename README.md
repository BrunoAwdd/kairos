# kairos-core

<!-- Badges -->

[![Crates.io](https://img.shields.io/crates/v/kairos-core.svg)](https://crates.io/crates/kairos-core)
[![docs.rs](https://img.shields.io/docsrs/kairos-core)](https://docs.rs/kairos-core)
[![Downloads](https://img.shields.io/crates/d/kairos-core.svg)](https://crates.io/crates/kairos-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![no_std](https://img.shields.io/badge/no__std-compatible-success)](#)

<!-- Replace USER/REPO and workflow file if you use GitHub Actions -->

[![CI](https://github.com/brunoAwdd/kratos/actions/workflows/ci.yml/badge.svg)](https://github.com/brunoAwdd/kratos/actions/workflows/ci.yml)

**Minimal, zero-dependency virtual clock & time utilities for Rust.**  
Supports `std` and `no_std` (embedded, WASM) with selectable backends.  
Designed for **deterministic simulation**, **testing**, and **high‑performance scheduling**.

---

## ✨ Features

- **Virtual time types**: `VInstant`, `VDuration` – nanosecond precision, safe arithmetic.
- **`Clock` trait** for pluggable time sources.
- Multiple clock implementations:
  - `ManualClock` – fully controlled, perfect for deterministic tests.
  - `RateClock` – time progression with configurable rate.
  - `AutoClock` – selects a backend via Cargo features:
    - `autoclock-soft` → portable atomic counter (`no_std`), _ultra fast reads_.
    - `autoclock-std` → wraps `std::time::Instant` (desktop/server).
    - `autoclock-systick` → ARM Cortex‑M SysTick polling (bare‑metal).
- **Hybrid Logical Clock (HLC)** – monotonic timestamps that merge physical & logical time.
- **Calendar** – civil date/time conversion (Howard Hinnant algorithms), allocation‑free.
- **Scheduler** – minimal event scheduler driven by any `Clock` implementation.
- **`no_std` friendly** – works on embedded, WASM, and host without heap by default.

---

## 📦 Installation

Crates.io (suggested):

```toml
[dependencies]
kairos-core = { version = "0.1", default-features = false, features = ["autoclock-soft"] }
```

> Pick _one_ clock backend feature: `autoclock-soft` (portable), `autoclock-std` (host), or `autoclock-systick` (Cortex‑M).

Local path (monorepo):

```toml
[dependencies]
kairos-core = { path = "./kairos-core", default-features = false, features = ["autoclock-soft"] }
```

**Optional features**

- `autoclock-soft` – portable atomic counter backend (**default in this repo**).
- `autoclock-std` – use `std::time::Instant`.
- `autoclock-systick` – ARM SysTick polling (no ISR).
- `bench-guards` – helpers for microbenchmarks.
- `alloc` – enable heap types in `no_std` environments.

---

## 🚀 Quick Start

```rust
use kairos_core::{AutoClock, Clock, VDuration};

fn main() {
    // AutoClock uses the selected backend (e.g., autoclock-soft)
    let mut clk = AutoClock::new();

    let t0 = clk.now();
    clk.advance(VDuration::from_millis(500));
    let dt = clk.now() - t0;

    println!("Elapsed (ns): {}", dt.0);
}
```

### With the scheduler (optional crate)

```rust
use kairos_core::{AutoClock, Clock, VDuration};
use kairos_scheduler::Scheduler;

fn main() {
    let mut sched = Scheduler::new(AutoClock::new());

    // fire in +500ms
    sched.schedule_in(VDuration::from_millis(500), "Hello, world!");

    // run until +1s
    let target = sched.now() + VDuration::from_secs(1);
    sched.run_until(target, |msg| {
        println!("event: {msg}");
    });
}
```

### Hybrid Logical Clock (HLC)

```rust
use kairos_core::{ManualClock, KairosHlc};

let mut hlc = KairosHlc::new(ManualClock::default(), /*node_id=*/1);
let a = hlc.now();
let b = hlc.now();
assert!(a < b); // monotonic locally
```

### Calendar formatting (no_std friendly)

```rust
use kairos_core::{Calendar, VInstant, VDuration};

let cal = Calendar::new()
    .with_epoch(1970, 1, 1)      // anchor VInstant(0) to 1970-01-01
    .with_tz_offset_secs(-3*3600); // UTC-03:00

let t = VInstant(0) + VDuration::from_secs(12*3600 + 34*60 + 56); // 12:34:56
let s = cal.format(t);
assert!(s.starts_with("1970-01-01T12:34:56.000-03:00"));
```

---

## 📊 Benchmarks

Run (Criterion):

```bash
# soft backend (portable)
cargo bench --features autoclock-soft

# std backend (host Instant)
cargo bench --features autoclock-std
```

Example results (x86_64, release):

| Operation            | Time (approx) |
| -------------------- | ------------- |
| `SoftClock::now()`   | **0.26 ns**   |
| `ManualClock::now()` | ~1.34 ns      |
| `Instant::now()`     | ~25 ns        |
| `SystemTime::now()`  | ~27 ns        |

> `autoclock-soft` is essentially an atomic `u64` load—ideal for simulations, tests, and deterministic replays.

---

## 🎯 Design Goals

- **Determinism**: reproducible runs independent of host timing.
- **Performance**: sub‑nanosecond reads with the soft backend.
- **Portability**: `no_std` first; optional `std` integration.
- **Small surface**: tiny, focused abstractions that compose.

---

## 🧪 Testing

```bash
cargo test
# or selecting a backend
cargo test --no-default-features --features autoclock-soft
```

---

## 🔧 Build Matrix (examples)

```bash
# Embedded (Cortex-M SysTick)
cargo build --target thumbv7em-none-eabihf --no-default-features --features autoclock-systick

# WASM (wasip1)
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --no-default-features --features autoclock-soft
```

---

## 📜 License

MIT. See [LICENSE](LICENSE.md).

---

## 🙌 Acknowledgements

- Howard Hinnant for civil date algorithms.
- The Rust community for `no_std` and embedded best practices.

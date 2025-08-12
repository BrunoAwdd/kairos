#![allow(clippy::manual_saturating_arithmetic)]

/// Monotonic virtual timestamp in **nanoseconds** since an arbitrary epoch.
/// Arithmetic is **wrapping** for deltas across the epoch boundary.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct VInstant(pub u64);

/// Virtual duration in **nanoseconds**.
/// Construction helpers are **saturating** to avoid overflow on large inputs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VDuration(pub u64);

impl VDuration {
    /// Create from whole seconds (saturating).
    #[inline(always)]
    pub const fn from_secs(s: u64) -> Self {
        Self(s.saturating_mul(1_000_000_000))
    }

    /// Create from whole milliseconds (saturating).
    #[inline(always)]
    pub const fn from_millis(ms: u64) -> Self {
        Self(ms.saturating_mul(1_000_000))
    }

    /// Create from whole microseconds (saturating).
    #[inline(always)]
    pub const fn from_micros(us: u64) -> Self {
        Self(us.saturating_mul(1_000))
    }

    /// Create from whole nanoseconds (identity).
    #[inline(always)]
    pub const fn from_nanos(ns: u64) -> Self { Self(ns) }

    /// Get the duration in nanoseconds (identity).
    #[inline(always)]
    pub const fn as_nanos(self) -> u64 { self.0 }

    /// Get the duration as seconds (floating-point). Handy for rendering/logging.
    #[inline(always)]
    pub fn as_secs_f64(self) -> f64 { (self.0 as f64) / 1_000_000_000.0 }
}

/* ---- Arithmetic between VInstant and VDuration ---- */

impl core::ops::Add<VDuration> for VInstant {
    type Output = VInstant;
    #[inline(always)]
    fn add(self, rhs: VDuration) -> Self::Output {
        VInstant(self.0.wrapping_add(rhs.0))
    }
}

impl core::ops::AddAssign<VDuration> for VInstant {
    #[inline(always)]
    fn add_assign(&mut self, rhs: VDuration) {
        self.0 = self.0.wrapping_add(rhs.0);
    }
}

impl core::ops::Sub<VInstant> for VInstant {
    type Output = VDuration;
    #[inline(always)]
    fn sub(self, rhs: VInstant) -> Self::Output {
        // Wrapping subtraction preserves correct deltas even across epoch wrap.
        VDuration(self.0.wrapping_sub(rhs.0))
    }
}

/* ---- Arithmetic on VDuration itself (useful for scaling) ---- */

impl core::ops::Add for VDuration {
    type Output = VDuration;
    #[inline(always)]
    fn add(self, rhs: VDuration) -> Self::Output {
        VDuration(self.0.saturating_add(rhs.0))
    }
}

impl core::ops::Sub for VDuration {
    type Output = VDuration;
    #[inline(always)]
    fn sub(self, rhs: VDuration) -> Self::Output {
        VDuration(self.0.saturating_sub(rhs.0))
    }
}

impl core::ops::AddAssign for VDuration {
    #[inline(always)]
    fn add_assign(&mut self, rhs: VDuration) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}

/* ---- Interop with core::time::Duration ---- */

impl From<core::time::Duration> for VDuration {
    #[inline(always)]
    fn from(d: core::time::Duration) -> Self {
        // Saturating: if u64 ns would overflow, clamp to u64::MAX.
        let ns = d.as_secs()
            .saturating_mul(1_000_000_000)
            .saturating_add(d.subsec_nanos() as u64);
        VDuration(ns)
    }
}

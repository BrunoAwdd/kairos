// kairos-core/src/calendar.rs
use crate::VInstant;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct CivilDays(i64); // days since 1970-01-01 (proleptic Gregorian)

#[derive(Clone, Copy, Debug)]
enum Anchor { Utc, Local }

#[cfg(feature = "std")]
type SmallString = std::string::String;

#[cfg(not(feature = "std"))]
type SmallString = heapless::String<32>;

// Howard Hinnant's date algorithms (adapted) — allocation-free, no_std-friendly.
#[inline(always)]
fn days_from_civil(mut y: i32, m: u32, d: u32) -> i64 {
    // return: days since 1970-01-01
    let d = d as i32;
    y -= (m <= 2) as i32;
    let era = (y as i64).div_euclid(400);
    let yoe = (y as i64 - era * 400) as i64;
    let m = m as i64;
    let doy = (153 * ((m + 9) % 12) + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

#[inline(always)]
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    // input: days since 1970-01-01
    let z = z + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }).div_euclid(146_097);
    let doe = z - era * 146_097;                      // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0,399]
    let mut y = (yoe + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;                     // [0,11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;    // [1,31]
    let m = (mp + if mp < 10 { 3 } else { -9 }) as u32; // [1,12]
    if m <= 2 { y += 1; }
    (y, m, d)
}

#[derive(Clone, Copy, Debug)]
pub struct Calendar {
    epoch_days: CivilDays,   // where VInstant(0) anchors in the calendar
    tz_offset_secs: i32,     // e.g., -10800 for UTC-03:00
    anchor: Anchor,
}
impl Default for Calendar {
    fn default() -> Self {
        Self {
            epoch_days: CivilDays(0), // 1970-01-01
            tz_offset_secs: 0,        // UTC
            anchor: Anchor::Utc
        }
    }
}
impl Calendar {
    pub fn new() -> Self { Self::default() }

    /// Set the civil date that corresponds to VInstant(0).
    pub fn with_epoch(mut self, year: i32, month: u32, day: u32) -> Self {
        self.epoch_days = CivilDays(days_from_civil(year, month, day));
        self.anchor = Anchor::Utc;
        self
    }
    
    pub fn with_epoch_local(mut self, y: i32, m: u32, d: u32) -> Self {
        self.epoch_days = CivilDays(days_from_civil(y, m, d));
        self.anchor = Anchor::Local;
        self
    }

    /// Set a fixed timezone offset (seconds relative to UTC), e.g., -3h = -10800.
    pub fn with_tz_offset_secs(mut self, offset: i32) -> Self {
        self.tz_offset_secs = offset;
        self
    }

    /// Convert a VInstant (ns) into (YYYY,MM,DD, hh,mm,ss, millis) in the selected timezone.
    pub fn to_civil(&self, t: VInstant) -> (i32, u32, u32, u32, u32, u32, u32) {
        // ns → seconds and remainder ns
        let total_ns = t.0 as i128;
        let mut total_s = (total_ns / 1_000_000_000) as i64;
        let sub_ns = (total_ns % 1_000_000_000) as i64;

        // apply timezone offset (may be negative)
        if let Anchor::Utc = self.anchor {
            total_s += self.tz_offset_secs as i64;
        }

        // split into days and seconds-of-day
        let secs_per_day = 86_400i64;
        let mut day = self.epoch_days.0 + total_s.div_euclid(secs_per_day);
        let mut sod = total_s.rem_euclid(secs_per_day); // 0..86399

        // correct for negatives (rare edge case)
        if sod < 0 {
            sod += secs_per_day;
            day -= 1;
        }

        let (year, month, dom) = civil_from_days(day);
        let hour = (sod / 3600) as u32;
        let min  = ((sod % 3600) / 60) as u32;
        let sec  = (sod % 60) as u32;
        let millis = (sub_ns / 1_000_000) as u32;
        (year, month, dom, hour, min, sec, millis)
    }

    /// Format as `YYYY-MM-DDTHH:MM:SS.mmm±HH:MM`
    pub fn format(&self, t: VInstant) -> SmallString {
        use core::fmt::Write;
        let (y, mo, d, h, mi, s, ms) = self.to_civil(t);

        let sign = if self.tz_offset_secs >= 0 { '+' } else { '-' };
        let off = self.tz_offset_secs.abs();
        let off_h = (off / 3600) as u32;
        let off_m = ((off % 3600) / 60) as u32;

        #[cfg(feature = "std")]
        let mut out: SmallString = SmallString::with_capacity(32);
    
        #[cfg(not(feature = "std"))]
        let mut out: SmallString = SmallString::new();
    
        let _ = write!(
            &mut out,
            "{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{ms:03}{sign}{off_h:02}:{off_m:02}"
        );
        out
    }
}

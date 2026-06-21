use pleiades_types::JulianDay;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CivilDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: f64,
}

impl CivilDateTime {
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: f64) -> Self {
        Self { year, month, day, hour, minute, second }
    }

    pub fn to_julian_day(&self) -> f64 {
        let day_frac = self.day as f64
            + (self.hour as f64 + self.minute as f64 / 60.0 + self.second / 3600.0) / 24.0;
        let (y, m) = if self.month <= 2 {
            (self.year - 1, self.month as i32 + 12)
        } else {
            (self.year, self.month as i32)
        };
        let a = (y as f64 / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();
        let jd = (365.25 * (y as f64 + 4716.0)).floor()
            + (30.6001 * (m as f64 + 1.0)).floor()
            + day_frac
            + b
            - 1524.5;
        jd
    }

    pub fn from_julian_day(jd: f64) -> Self {
        let jd = jd + 0.5;
        let z = jd.floor();
        let f = jd - z;
        let alpha = ((z - 1867216.25) / 36524.25).floor();
        let a = z + 1.0 + alpha - (alpha / 4.0).floor();
        let b = a + 1524.0;
        let c = ((b - 122.1) / 365.25).floor();
        let d = (365.25 * c).floor();
        let e = ((b - d) / 30.6001).floor();
        let day_frac = b - d - (30.6001 * e).floor() + f;
        let day = day_frac.floor();
        let month = if e < 14.0 { e - 1.0 } else { e - 13.0 };
        let year = if month > 2.0 { c - 4716.0 } else { c - 4715.0 };
        let mut rem_hours = (day_frac - day) * 24.0;
        let hour = rem_hours.floor();
        rem_hours = (rem_hours - hour) * 60.0;
        let minute = rem_hours.floor();
        let second = (rem_hours - minute) * 60.0;
        Self {
            year: year as i32,
            month: month as u8,
            day: day as u8,
            hour: hour as u8,
            minute: minute as u8,
            second,
        }
    }
}

fn main() {
    let original = CivilDateTime::new(1987, 4, 10, 19, 21, 0.0);
    let jd = original.to_julian_day();
    println!("Forward JD: {}", jd);
    
    let back = CivilDateTime::from_julian_day(jd);
    println!("Back: {}y {}m {}d {}h {}m {}s", back.year, back.month, back.day, back.hour, back.minute, back.second);
    
    println!("\nAssertions:");
    println!("year == 1987: {}", back.year == 1987);
    println!("month == 4: {}", back.month == 4);
    println!("day == 10: {}", back.day == 10);
    println!("hour == 19: {}", back.hour == 19);
    println!("minute == 21: {} (got {})", back.minute == 21, back.minute);
    println!("second < 0.001: {} (got {})", back.second < 0.001, back.second);
    println!("second > 59.999: {} (got {})", back.second > 59.999, back.second);
    println!("second < 0.001 || second > 59.999: {}", back.second < 0.001 || back.second > 59.999);
}

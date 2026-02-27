// Time module stubs for JIT
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use std::thread;
use std::time::Duration;

pub extern "C" fn vp_time_time() -> f64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs_f64(),
        Err(_) => 0.0,
    }
}

pub extern "C" fn vp_time_monotonic() -> f64 {
    thread_local! {
        static START: Instant = Instant::now();
    }
    START.with(|start| start.elapsed().as_secs_f64())
}

pub extern "C" fn vp_time_perf_counter() -> f64 {
    thread_local! {
        static START: Instant = Instant::now();
    }
    START.with(|start| start.elapsed().as_secs_f64())
}

pub extern "C" fn vp_time_sleep(seconds: f64) {
    if seconds > 0.0 {
        let duration = Duration::from_secs_f64(seconds);
        thread::sleep(duration);
    }
}

pub extern "C" fn vp_time_localtime(
    timestamp: f64,
    year: *mut i64,
    month: *mut i64,
    day: *mut i64,
    hour: *mut i64,
    minute: *mut i64,
    second: *mut i64,
) {
    let secs = timestamp as u64;
    let _datetime = UNIX_EPOCH + Duration::from_secs(secs);
    
    // Convert to system time and extract components
    // This is a simplified implementation
    let days_since_epoch = secs / 86400;
    let remaining_secs = secs % 86400;
    
    let h = (remaining_secs / 3600) as i64;
    let m = ((remaining_secs % 3600) / 60) as i64;
    let s = (remaining_secs % 60) as i64;
    
    // Simplified date calculation
    let mut days = days_since_epoch as i64;
    let mut y = 1970i64;
    
    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        y += 1;
    }
    
    let mut m_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if is_leap_year(y) {
        m_days[1] = 29;
    }
    
    let mut month_idx = 0;
    while days >= m_days[month_idx] as i64 && month_idx < 11 {
        days -= m_days[month_idx] as i64;
        month_idx += 1;
    }
    
    let d = days + 1;
    let mo = (month_idx + 1) as i64;
    
    unsafe {
        if !year.is_null() { *year = y; }
        if !month.is_null() { *month = mo; }
        if !day.is_null() { *day = d; }
        if !hour.is_null() { *hour = h; }
        if !minute.is_null() { *minute = m; }
        if !second.is_null() { *second = s; }
    }
}

pub extern "C" fn vp_time_gmtime(
    timestamp: f64,
    year: *mut i64,
    month: *mut i64,
    day: *mut i64,
    hour: *mut i64,
    minute: *mut i64,
    second: *mut i64,
) {
    // For simplicity, same as localtime (UTC handling would require external crate)
    vp_time_localtime(timestamp, year, month, day, hour, minute, second);
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub extern "C" fn vp_time_strftime(timestamp: f64, format: *const i8) -> *mut i8 {
    let fmt_str = if format.is_null() {
        "%Y-%m-%d %H:%M:%S"
    } else {
        unsafe {
            std::ffi::CStr::from_ptr(format).to_str().unwrap_or("%Y-%m-%d %H:%M:%S")
        }
    };
    
    let secs = timestamp as u64;
    let days_since_epoch = secs / 86400;
    let remaining_secs = secs % 86400;
    
    let h = (remaining_secs / 3600) as i64;
    let m = ((remaining_secs % 3600) / 60) as i64;
    let s = (remaining_secs % 60) as i64;
    
    // Simplified date calculation
    let mut days = days_since_epoch as i64;
    let mut y = 1970i64;
    
    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        y += 1;
    }
    
    let mut m_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if is_leap_year(y) {
        m_days[1] = 29;
    }
    
    let mut month_idx = 0;
    while days >= m_days[month_idx] as i64 && month_idx < 11 {
        days -= m_days[month_idx] as i64;
        month_idx += 1;
    }
    
    let d = days + 1;
    let mo = (month_idx + 1) as i64;
    
    // Simple format substitution
    let result = fmt_str
        .replace("%Y", &y.to_string())
        .replace("%m", &format!("{:02}", mo))
        .replace("%d", &format!("{:02}", d))
        .replace("%H", &format!("{:02}", h))
        .replace("%M", &format!("{:02}", m))
        .replace("%S", &format!("{:02}", s));
    
    let c_str = std::ffi::CString::new(result).unwrap();
    c_str.into_raw()
}

pub extern "C" fn vp_time_timezone_offset() -> i64 {
    // Return 0 for UTC (proper TZ handling would require external crate)
    0
}

pub extern "C" fn vp_time_isdst() -> i64 {
    // DST detection would require external crate
    0
}

pub extern "C" fn vp_time_days_in_month(year: i64, month: i64) -> i64 {
    if month < 1 || month > 12 {
        return 30;
    }
    
    let days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    
    if month == 2 && is_leap_year(year) {
        return 29;
    }
    
    days[month as usize]
}

pub extern "C" fn vp_time_sleep_ms(milliseconds: i64) {
    if milliseconds > 0 {
        let duration = Duration::from_millis(milliseconds as u64);
        thread::sleep(duration);
    }
}

pub extern "C" fn vp_time_sleep_us(microseconds: i64) {
    if microseconds > 0 {
        let duration = Duration::from_micros(microseconds as u64);
        thread::sleep(duration);
    }
}

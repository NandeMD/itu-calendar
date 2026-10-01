use crate::CourseMeeting;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

const ISTANBUL_OFFSET_SECONDS: i64 = 3 * 60 * 60;

/// Build a downloadable iCalendar document for meetings with usable day and time data.
/// Meetings repeat weekly because this app's course feed has no semester date range.
pub fn serialize_ical(meetings: &[CourseMeeting], english: bool) -> Option<String> {
    let now = current_unix_seconds()?;
    let istanbul_now = now + ISTANBUL_OFFSET_SECONDS;
    let today = istanbul_now.div_euclid(86_400);
    let minute_now = istanbul_now.rem_euclid(86_400) / 60;
    let timestamp = format_utc(now);

    let mut lines = vec![
        "BEGIN:VCALENDAR".to_owned(),
        "VERSION:2.0".to_owned(),
        "PRODID:-//ITU Calendar//Course Schedule//EN".to_owned(),
        "CALSCALE:GREGORIAN".to_owned(),
        format!("X-WR-CALNAME:{}", escape_text(if english { "ITU Course Schedule" } else { "İTÜ Ders Programı" })),
        "BEGIN:VTIMEZONE".to_owned(),
        "TZID:Europe/Istanbul".to_owned(),
        "X-LIC-LOCATION:Europe/Istanbul".to_owned(),
        "BEGIN:STANDARD".to_owned(),
        "DTSTART:19700101T000000".to_owned(),
        "TZOFFSETFROM:+0300".to_owned(),
        "TZOFFSETTO:+0300".to_owned(),
        "TZNAME:TRT".to_owned(),
        "END:STANDARD".to_owned(),
        "END:VTIMEZONE".to_owned(),
    ];

    let mut event_count = 0;
    for meeting in meetings {
        let Some((by_day, weekday)) = parse_weekday(&meeting.weekday) else { continue };
        let Some(start_minute) = parse_time(&meeting.start_time) else { continue };
        let Some(end_minute) = parse_time(&meeting.end_time) else { continue };
        if end_minute < start_minute { continue }

        let days_until = (weekday - (today + 4).rem_euclid(7) as i64).rem_euclid(7);
        let days_until = if days_until == 0 && minute_now >= start_minute { 7 } else { days_until };
        let event_day = today + days_until;
        let start = format_local(event_day, start_minute);
        // OBS end times include the final minute (for example 11:29), while iCalendar DTEND is exclusive.
        let end = format_local(event_day, end_minute + 1);
        let title = format!("{} · CRN {}", meeting.course_name, meeting.crn);
        let mut description = Vec::new();
        if !meeting.instructor.trim().is_empty() && meeting.instructor.trim() != "[]" {
            description.push(format!("{}: {}", if english { "Instructor" } else { "Öğretim görevlisi" }, meeting.instructor.trim()));
        }
        if !meeting.teaching_method.trim().is_empty() && meeting.teaching_method.trim() != "[]" {
            description.push(format!("{}: {}", if english { "Teaching method" } else { "Öğretim türü" }, meeting.teaching_method.trim()));
        }

        lines.extend([
            "BEGIN:VEVENT".to_owned(),
            format!("UID:crn-{}-{}-{}@itu-calendar", escape_text(&meeting.crn), by_day, start_minute),
            format!("DTSTAMP:{timestamp}"),
            format!("DTSTART;TZID=Europe/Istanbul:{start}"),
            format!("DTEND;TZID=Europe/Istanbul:{end}"),
            format!("RRULE:FREQ=WEEKLY;BYDAY={by_day}"),
            format!("SUMMARY:{}", escape_text(&title)),
        ]);
        if !description.is_empty() {
            lines.push(format!("DESCRIPTION:{}", escape_text(&description.join("\n"))));
        }
        if let Some(location) = meeting.location.as_deref().filter(|location| !location.trim().is_empty()) {
            lines.push(format!("LOCATION:{}", escape_text(location.trim())));
        }
        lines.extend(["STATUS:CONFIRMED".to_owned(), "TRANSP:OPAQUE".to_owned(), "END:VEVENT".to_owned()]);
        event_count += 1;
    }

    if event_count == 0 { return None; }
    lines.push("END:VCALENDAR".to_owned());
    Some(format!("{}\r\n", lines.iter().map(|line| fold_line(line)).collect::<Vec<_>>().join("\r\n")))
}

#[cfg(target_arch = "wasm32")]
fn current_unix_seconds() -> Option<i64> {
    let milliseconds = js_sys::Date::now();
    milliseconds.is_finite().then_some((milliseconds / 1000.0) as i64)
}

#[cfg(not(target_arch = "wasm32"))]
fn current_unix_seconds() -> Option<i64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs() as i64)
}

/// Percent-encode UTF-8 bytes for a data URL, keeping only URI unreserved characters.
pub fn data_url(calendar: &str) -> String {
    let mut url = String::from("data:text/calendar;charset=utf-8,");
    for byte in calendar.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            url.push(byte as char);
        } else {
            url.push_str(&format!("%{byte:02X}"));
        }
    }
    url
}

fn parse_weekday(day: &str) -> Option<(&'static str, i64)> {
    match day.trim().to_lowercase().as_str() {
        "pazartesi" | "monday" => Some(("MO", 1)),
        "salı" | "sali" | "tuesday" => Some(("TU", 2)),
        "çarşamba" | "carsamba" | "wednesday" => Some(("WE", 3)),
        "perşembe" | "persembe" | "thursday" => Some(("TH", 4)),
        "cuma" | "friday" => Some(("FR", 5)),
        "cumartesi" | "saturday" => Some(("SA", 6)),
        "pazar" | "sunday" => Some(("SU", 0)),
        _ => None,
    }
}

fn parse_time(time: &str) -> Option<i64> {
    let mut parts = time.trim().split(':');
    let hour = parts.next()?.parse::<i64>().ok()?;
    let minute = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some() || !(0..24).contains(&hour) || !(0..60).contains(&minute) {
        return None;
    }
    Some(hour * 60 + minute)
}

fn format_local(day: i64, minute: i64) -> String {
    let day = day + minute.div_euclid(1440);
    let minute = minute.rem_euclid(1440);
    let (year, month, date) = civil_from_days(day);
    format!("{year:04}{month:02}{date:02}T{:02}{:02}00", minute / 60, minute % 60)
}

fn format_utc(seconds: i64) -> String {
    let day = seconds.div_euclid(86_400);
    let second_of_day = seconds.rem_euclid(86_400);
    let (year, month, date) = civil_from_days(day);
    format!("{year:04}{month:02}{date:02}T{:02}{:02}{:02}Z", second_of_day / 3600, second_of_day / 60 % 60, second_of_day % 60)
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let date = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    (year, month, date)
}

fn escape_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\r', "")
        .replace('\n', "\\n")
        .replace(',', "\\,")
        .replace(';', "\\;")
}

fn fold_line(line: &str) -> String {
    let mut folded = String::new();
    let mut segment_bytes = 0;
    for character in line.chars() {
        let character_bytes = character.len_utf8();
        if segment_bytes + character_bytes > 75 {
            folded.push_str("\r\n ");
            segment_bytes = 1;
        }
        folded.push(character);
        segment_bytes += character_bytes;
    }
    folded
}

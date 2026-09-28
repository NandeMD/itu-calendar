use crate::CourseMeeting;

/// Serialize resolved meetings as an iCalendar (.ics) document.
pub fn serialize_ical(_meetings: &[CourseMeeting]) -> String {
    // TODO: Add iCalendar serialization when schedule data is available.
    String::new()
}

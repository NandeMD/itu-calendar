use crate::CourseMeeting;
use dioxus::prelude::ServerFnError;
#[cfg(feature = "server")]
use futures_util::{StreamExt, TryStreamExt};
use serde::Deserialize;

const LESSONS_URL: &str = "https://raw.githubusercontent.com/itu-helper/data/main/lessons.psv";

#[derive(Deserialize)]
pub struct Lesson {
    pub crn: u32,
    pub course_code: String,
    pub teaching_method: String,
    pub instructor: String,
    pub buildings: String,
    pub day: String,
    pub time: String,
    pub room: String,
    pub capacity: u32,
    pub enrolled: u32,
    pub majors: String,
}

/// Fetch, parse, and persist the upstream lesson export.
#[cfg(feature = "server")]
pub async fn sync_lessons() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = reqwest::get(LESSONS_URL).await?.error_for_status()?;
    let body =
        tokio_util::io::StreamReader::new(response.bytes_stream().map_err(std::io::Error::other));

    let mut reader = csv_async::AsyncReaderBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .create_deserializer(body)
        .into_deserialize::<Lesson>();

    const BATCH_SIZE: usize = 256;
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    while let Some(lesson) = reader.next().await {
        batch.push(lesson?);
        if batch.len() == BATCH_SIZE {
            crate::db::upsert_lessons(&batch)?;
            batch.clear();
        }
    }
    if !batch.is_empty() {
        crate::db::upsert_lessons(&batch)?;
    }
    Ok(())
}

/// Server-side boundary for resolving lecture CRNs against ITU OBS.
#[cfg(feature = "server")]
pub async fn lookup_crns(crns: Vec<String>) -> Result<Vec<CourseMeeting>, ServerFnError> {
    let crns = crns
        .into_iter()
        .map(|crn| {
            crn.trim()
                .parse::<u32>()
                .map_err(|_| ServerFnError::new(format!("Invalid CRN: {crn}")))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let lessons = crate::db::find_lessons_by_crn(crns)
        .map_err(|error| ServerFnError::new(error.to_string()))?;

    Ok(lessons
        .into_iter()
        .map(|lesson| {
            let (start_time, end_time) = lesson.time.split_once('/').unwrap_or((&lesson.time, ""));
            let buildings = lesson.buildings.trim();
            let room = lesson.room.trim();
            let location = match (buildings, room) {
                ("", "") => None,
                (buildings, "" | "--") => Some(buildings.to_owned()),
                ("", room) => Some(room.to_owned()),
                (buildings, room) => Some(format!("{buildings} {room}")),
            };

            CourseMeeting {
                crn: lesson.crn.to_string(),
                course_name: lesson.course_code,
                weekday: lesson.day,
                start_time: start_time.to_owned(),
                end_time: end_time.to_owned(),
                location,
            }
        })
        .collect())
}

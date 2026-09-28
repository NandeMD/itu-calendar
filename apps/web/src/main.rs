mod calendar;
mod db;
mod obs;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        obs::sync_lessons()
            .await
            .expect("Failed to sync lesson data before starting the server");

        Ok(dioxus::server::router(app))
    });

    #[cfg(not(feature = "server"))]
    dioxus::launch(app);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CourseMeeting {
    crn: String,
    course_name: String,
    weekday: String,
    start_time: String,
    end_time: String,
    location: Option<String>,
}

#[server]
async fn lookup_courses(crns: Vec<String>) -> Result<Vec<CourseMeeting>, ServerFnError> {
    obs::lookup_crns(crns).await
}

fn app() -> Element {
    let mut crns = use_signal(String::new);
    let mut meetings = use_signal(Vec::<CourseMeeting>::new);
    let mut has_searched = use_signal(|| false);
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    rsx! {
        main {
            h1 { "ITU Calendar" }
            p { "Build a course calendar by entering one or more lecture CRNs." }
            form {
                onsubmit: move |event: FormEvent| async move {
                    event.prevent_default();

                    let requested_crns = crns
                        .read()
                        .split(|character: char| character == ',' || character.is_whitespace())
                        .filter(|crn| !crn.is_empty())
                        .map(str::to_owned)
                        .collect::<Vec<_>>();

                    if requested_crns.is_empty() {
                        error.set(Some("Enter at least one CRN.".to_owned()));
                        meetings.set(Vec::new());
                        has_searched.set(false);
                        return;
                    }

                    loading.set(true);
                    error.set(None);
                    has_searched.set(false);

                    match lookup_courses(requested_crns).await {
                        Ok(found_meetings) => {
                            meetings.set(found_meetings);
                            has_searched.set(true);
                        }
                        Err(lookup_error) => {
                            error.set(Some(lookup_error.to_string()));
                            meetings.set(Vec::new());
                        }
                    }

                    loading.set(false);
                },
                label {
                    r#for: "crns",
                    "Lecture CRNs"
                }
                textarea {
                    id: "crns",
                    name: "crns",
                    placeholder: "Enter CRNs separated by commas or spaces",
                    value: "{crns}",
                    oninput: move |event| crns.set(event.value()),
                }
                button {
                    r#type: "submit",
                    disabled: loading(),
                    if loading() { "Looking up…" } else { "Check Lessons" }
                }
            }

            if let Some(message) = error.read().as_ref() {
                p { role: "alert", "{message}" }
            }

            if loading() {
                p { "Looking up lessons…" }
            }

            if has_searched() {
                if meetings.read().is_empty() {
                    p { "No lessons found for those CRNs." }
                } else {
                    table {
                        thead {
                            tr {
                                th { "CRN" }
                                th { "Course" }
                                th { "Day" }
                                th { "Time" }
                                th { "Location" }
                            }
                        }
                        tbody {
                            for (index, meeting) in meetings.read().iter().enumerate() {
                                tr { key: "{index}-{meeting.crn}",
                                    td { "{meeting.crn}" }
                                    td { "{meeting.course_name}" }
                                    td { "{meeting.weekday}" }
                                    td { "{meeting.start_time}–{meeting.end_time}" }
                                    td { "{meeting.location.as_deref().unwrap_or(\"—\")}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

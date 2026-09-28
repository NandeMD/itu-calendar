mod calendar;
mod obs;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

fn main() {
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
    rsx! {
        main {
            h1 { "ITU Calendar" }
            p { "Build a course calendar by entering one or more lecture CRNs." }
            form {
                label {
                    r#for: "crns",
                    "Lecture CRNs"
                }
                textarea {
                    id: "crns",
                    name: "crns",
                    placeholder: "Enter CRNs separated by commas or spaces",
                    rows: "5",
                }
                button { r#type: "submit", "Create calendar" }
            }
            p { "CRN lookup and iCalendar export are not connected yet." }
        }
    }
}

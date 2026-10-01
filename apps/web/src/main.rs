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
    teaching_method: String,
    instructor: String,
    building: String,
    room: String,
    capacity: u32,
    enrolled: u32,
    majors: String,
}

#[derive(Clone, Copy)]
enum AppError {
    MissingCrn,
    LookupFailed,
}

fn localized_weekday(weekday: &str, english: bool) -> String {
    let normalized = weekday.trim().to_lowercase();
    let translated = match (english, normalized.as_str()) {
        (true, "pazartesi") => "Monday",
        (true, "salı") => "Tuesday",
        (true, "çarşamba") => "Wednesday",
        (true, "perşembe") => "Thursday",
        (true, "cuma") => "Friday",
        (true, "cumartesi") => "Saturday",
        (true, "pazar") => "Sunday",
        (false, "monday") => "Pazartesi",
        (false, "tuesday") => "Salı",
        (false, "wednesday") => "Çarşamba",
        (false, "thursday") => "Perşembe",
        (false, "friday") => "Cuma",
        (false, "saturday") => "Cumartesi",
        (false, "sunday") => "Pazar",
        _ => return weekday.to_owned(),
    };
    translated.to_owned()
}

fn detail_or_dash(value: &str) -> &str {
    let value = value.trim();
    if value.is_empty() || value == "[]" {
        "—"
    } else {
        value
    }
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
    let mut error = use_signal(|| None::<AppError>);
    let mut selected_meeting = use_signal(|| None::<usize>);
    let mut language = use_signal(|| "tr".to_owned());
    let english = language() == "en";
    let calendar_download = calendar::serialize_ical(meetings.read().as_slice(), english)
        .map(|calendar| calendar::data_url(&calendar));

    rsx! {
        document::Link {
            rel: "icon",
            href: asset!("/assets/favicon.svg"),
            r#type: "image/svg+xml",
        }
        document::Link {
            rel: "stylesheet",
            href: "https://cdn.jsdelivr.net/npm/daisyui@5",
        }
        document::Link {
            rel: "stylesheet",
            href: "https://cdn.jsdelivr.net/npm/daisyui@5/themes.css",
        }
        document::Stylesheet { href: asset!("/assets/main.css") }

        main {
            class: "app-shell",
            "data-theme": "luxury",
            lang: "{language}",
            section { class: "content-wrap",
                div { class: "section-heading",
                    div {
                        p { class: "section-kicker", if english { "START WITH YOUR COURSES" } else { "DERSLERİNİZLE BAŞLAYIN" } }
                        h2 { if english { "Build your schedule" } else { "Ders programını oluştur" } }
                    }
                    div {
                        class: "language-toggle",
                        role: "group",
                        "aria-label": if english { "Language" } else { "Dil" },
                        button {
                            class: if english { "language-option" } else { "language-option active" },
                            r#type: "button",
                            "aria-pressed": !english,
                            onclick: move |_| language.set("tr".to_owned()),
                            "Türkçe"
                        }
                        button {
                            class: if english { "language-option active" } else { "language-option" },
                            r#type: "button",
                            "aria-pressed": english,
                            onclick: move |_| language.set("en".to_owned()),
                            "English"
                        }
                    }
                }

                form {
                    class: "lookup-card card",
                    onsubmit: move |event: FormEvent| async move {
                        event.prevent_default();

                        let requested_crns = crns
                            .read()
                            .split(|character: char| character == ',' || character.is_whitespace())
                            .filter(|crn| !crn.is_empty())
                            .map(str::to_owned)
                            .collect::<Vec<_>>();

                        if requested_crns.is_empty() {
                            error.set(Some(AppError::MissingCrn));
                            meetings.set(Vec::new());
                            has_searched.set(false);
                            return;
                        }

                        loading.set(true);
                        error.set(None);
                        has_searched.set(false);
                        selected_meeting.set(None);

                        match lookup_courses(requested_crns).await {
                            Ok(found_meetings) => {
                                meetings.set(found_meetings);
                                has_searched.set(true);
                            }
                            Err(_) => {
                                error.set(Some(AppError::LookupFailed));
                                meetings.set(Vec::new());
                            }
                        }

                        loading.set(false);
                    },
                    div { class: "form-topline",
                        div { class: "step-number", "01" }
                        div {
                            h3 { if english { "Add your lecture CRNs" } else { "Ders CRN'lerini ekle" } }
                            p { if english { "Look up one course or add several at once." } else { "Tek bir dersi arayın veya aynı anda birkaç ders ekleyin." } }
                        }
                    }
                    label { class: "field-label label", r#for: "crns", if english { "COURSE REGISTRATION NUMBERS (CRN)" } else { "DERS KAYIT NUMARALARI (CRN)" } }
                    textarea {
                        class: "crn-input textarea textarea-bordered",
                        id: "crns",
                        name: "crns",
                        placeholder: "e.g. 12345, 23456, 34567",
                        value: "{crns}",
                        oninput: move |event| crns.set(event.value()),
                    }
                    div { class: "form-bottomline",
                        p { class: "input-hint", if english { "Separate CRNs with commas or spaces." } else { "CRN'leri virgülle veya boşlukla ayırın." } }
                        button {
                            class: "lookup-button btn btn-primary",
                            r#type: "submit",
                            disabled: loading(),
                            if loading() {
                                span { class: "loading loading-spinner loading-sm" }
                                if english { "Searching" } else { "Aranıyor" }
                            } else {
                                if english { "Find my courses" } else { "Derslerimi bul" }
                                span { class: "button-arrow", "↗" }
                            }
                        }
                    }
                }

                if let Some(error_kind) = error() {
                    div { class: "status-alert alert alert-error", role: "alert",
                        span { class: "status-icon", "!" }
                        span {
                            match error_kind {
                                AppError::MissingCrn => if english { "Enter at least one CRN." } else { "En az bir CRN girin." },
                                AppError::LookupFailed => if english { "Could not find courses. Please try again." } else { "Dersler bulunamadı. Lütfen tekrar deneyin." },
                            }
                        }
                    }
                }

                if loading() {
                    div { class: "status-alert alert search-status", role: "status",
                        span { class: "loading loading-spinner loading-sm" }
                        span { if english { "Finding your course meetings…" } else { "Ders programınız aranıyor…" } }
                    }
                }

                if has_searched() {
                    if meetings.read().is_empty() {
                        div { class: "empty-state card",
                            span { class: "empty-mark", "—" }
                            h3 { if english { "No courses found" } else { "Ders bulunamadı" } }
                            p { if english { "Double-check the CRNs and try again." } else { "CRN numaralarını kontrol edip tekrar deneyin." } }
                        }
                    } else {
                        section { class: "results-section",
                            div { class: "results-heading",
                                div {
                                    p { class: "section-kicker", if english { "YOUR WEEK AT A GLANCE" } else { "HAFTALIK DERS PROGRAMINIZ" } }
                                    h2 { if english { "Course meetings" } else { "Ders saatleri" } }
                                }
                                div { class: "results-actions",
                                    if let Some(download_url) = calendar_download.as_deref() {
                                        a {
                                            class: "calendar-download btn btn-outline",
                                            href: "{download_url}",
                                            download: "itu-course-schedule.ics",
                                            if english { "Download calendar" } else { "Takvimi indir" }
                                        }
                                    }
                                    span { class: "result-count badge badge-neutral",
                                        "{meetings.read().len()}"
                                        if english { " FOUND" } else { " BULUNDU" }
                                    }
                                }
                            }
                            div { class: "table-frame card",
                                div { class: "table-scroll",
                                    table { class: "results-table table table-zebra",
                                        thead {
                                            tr {
                                                th { "CRN" }
                                                th { if english { "COURSE" } else { "DERS" } }
                                                th { if english { "DAY" } else { "GÜN" } }
                                                th { if english { "TIME" } else { "SAAT" } }
                                                th { if english { "LOCATION" } else { "YER" } }
                                            }
                                        }
                                        tbody {
                                            for (index, meeting) in meetings.read().iter().enumerate() {
                                                tr { key: "{index}-{meeting.crn}",
                                                    td { class: "crn-cell",
                                                        button {
                                                            class: "crn-link",
                                                            r#type: "button",
                                                            onclick: move |_| selected_meeting.set(Some(index)),
                                                            "{meeting.crn}"
                                                        }
                                                    }
                                                    td { class: "course-cell", "{meeting.course_name}" }
                                                    td { span { class: "day-chip badge badge-outline", "{localized_weekday(&meeting.weekday, english)}" } }
                                                    td { "{meeting.start_time}–{meeting.end_time}" }
                                                    td { class: "location-cell", "{meeting.location.as_deref().unwrap_or(\"—\")}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(index) = selected_meeting() {
                    if let Some(meeting) = meetings.read().get(index) {
                        div { class: "lesson-modal-backdrop",
                            div {
                                class: "lesson-modal card",
                                role: "dialog",
                                "aria-modal": "true",
                                "aria-labelledby": "lesson-dialog-title",
                                div { class: "lesson-modal-heading",
                                    div {
                                        p { class: "section-kicker", if english { "COURSE DETAILS" } else { "DERS DETAYLARI" } }
                                        h2 { id: "lesson-dialog-title", "{meeting.course_name}" }
                                        p { class: "lesson-modal-crn", "CRN {meeting.crn}" }
                                    }
                                    button {
                                        class: "lesson-modal-close",
                                        r#type: "button",
                                        "aria-label": if english { "Close details" } else { "Detayları kapat" },
                                        onclick: move |_| selected_meeting.set(None),
                                        "×"
                                    }
                                }
                                div { class: "lesson-details-grid",
                                    div { class: "lesson-detail",
                                        span { class: "lesson-detail-label", if english { "DAY" } else { "GÜN" } }
                                        span { class: "lesson-detail-value", "{localized_weekday(&meeting.weekday, english)}" }
                                    }
                                    div { class: "lesson-detail",
                                        span { class: "lesson-detail-label", if english { "TIME" } else { "SAAT" } }
                                        span { class: "lesson-detail-value", "{meeting.start_time}–{meeting.end_time}" }
                                    }
                                    div { class: "lesson-detail",
                                        span { class: "lesson-detail-label", if english { "TEACHING METHOD" } else { "ÖĞRETİM TÜRÜ" } }
                                        span { class: "lesson-detail-value", "{detail_or_dash(&meeting.teaching_method)}" }
                                    }
                                    div { class: "lesson-detail",
                                        span { class: "lesson-detail-label", if english { "INSTRUCTOR" } else { "ÖĞRETİM GÖREVLİSİ" } }
                                        span { class: "lesson-detail-value", "{detail_or_dash(&meeting.instructor)}" }
                                    }
                                    div { class: "lesson-detail",
                                        span { class: "lesson-detail-label", if english { "BUILDING" } else { "BİNA" } }
                                        span { class: "lesson-detail-value", "{detail_or_dash(&meeting.building)}" }
                                    }
                                    div { class: "lesson-detail",
                                        span { class: "lesson-detail-label", if english { "ROOM" } else { "DERSLİK" } }
                                        span { class: "lesson-detail-value", "{detail_or_dash(&meeting.room)}" }
                                    }
                                    div { class: "lesson-detail",
                                        span { class: "lesson-detail-label", if english { "ENROLLMENT" } else { "KAYIT DURUMU" } }
                                        span { class: "lesson-detail-value", "{meeting.enrolled} / {meeting.capacity}" }
                                    }
                                    div { class: "lesson-detail",
                                        span { class: "lesson-detail-label", if english { "AVAILABLE SEATS" } else { "KALAN KONTENJAN" } }
                                        span { class: "lesson-detail-value", "{meeting.capacity.saturating_sub(meeting.enrolled)}" }
                                    }
                                    div { class: "lesson-detail lesson-detail-wide",
                                        span { class: "lesson-detail-label", if english { "ELIGIBLE MAJORS" } else { "UYGUN BÖLÜMLER" } }
                                        span { class: "lesson-detail-value", "{detail_or_dash(&meeting.majors)}" }
                                    }
                                }
                            }
                        }
                    }
                }

            }
        }
    }
}

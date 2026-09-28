use crate::CourseMeeting;
use dioxus::prelude::ServerFnError;

/// Server-side boundary for resolving lecture CRNs against ITU OBS.
pub async fn lookup_crns(_crns: Vec<String>) -> Result<Vec<CourseMeeting>, ServerFnError> {
    // TODO: Implement OBS lookup after identifying its supported data endpoint.
    Ok(Vec::new())
}

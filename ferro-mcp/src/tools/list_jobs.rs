//! List jobs tool - scan for background jobs

use crate::error::Result;
use crate::introspection::jobs;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct JobsInfo {
    pub jobs: Vec<JobInfo>,
}

#[derive(Debug, Serialize)]
pub struct JobInfo {
    pub name: String,
    pub path: String,
    pub queue: Option<String>,
}

pub fn execute(project_root: &Path) -> Result<JobsInfo> {
    let jobs_info = jobs::scan_jobs(project_root);
    Ok(JobsInfo { jobs: jobs_info })
}

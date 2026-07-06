//! Job introspection

use crate::tools::list_jobs::JobInfo;
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{Attribute, ItemStruct};
use walkdir::WalkDir;

struct JobVisitor {
    jobs: Vec<String>,
}

impl JobVisitor {
    fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    fn has_job_derive(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) {
                    for path in nested {
                        let ident = path.segments.last().map(|s| s.ident.to_string());
                        if matches!(ident.as_deref(), Some("Job") | Some("Dispatchable")) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

impl<'ast> Visit<'ast> for JobVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if self.has_job_derive(&node.attrs) {
            self.jobs.push(node.ident.to_string());
        }
        syn::visit::visit_item_struct(self, node);
    }
}

pub fn scan_jobs(project_root: &Path) -> Vec<JobInfo> {
    let jobs_dir = project_root.join("src/jobs");

    let mut all_jobs = Vec::new();

    if !jobs_dir.exists() {
        return all_jobs;
    }

    for entry in WalkDir::new(&jobs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Ok(syntax) = syn::parse_file(&content) {
                let mut visitor = JobVisitor::new();
                visitor.visit_file(&syntax);

                let relative_path = entry
                    .path()
                    .strip_prefix(project_root)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();

                // Try to extract queue name from the file content
                let queue = extract_queue_name(&content);

                for job_name in visitor.jobs {
                    all_jobs.push(JobInfo {
                        name: job_name,
                        path: relative_path.clone(),
                        queue: queue.clone(),
                    });
                }
            }
        }
    }

    all_jobs
}

fn extract_queue_name(content: &str) -> Option<String> {
    // Look for queue() method implementation
    for line in content.lines() {
        if line.contains("fn queue") && line.contains("->") {
            // Try to find the return value
            if let Some(idx) = content.find("fn queue") {
                let snippet = &content[idx..];
                if let Some(quote_start) = snippet.find('"') {
                    let rest = &snippet[quote_start + 1..];
                    if let Some(quote_end) = rest.find('"') {
                        return Some(rest[..quote_end].to_string());
                    }
                }
            }
        }
    }
    None
}

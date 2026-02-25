use std::process::Command;

/// Clean build artifacts. Runs `cargo clean` by default.
/// With `--sweep`, removes only artifacts older than N days (requires cargo-sweep).
pub fn run(sweep_days: Option<u32>) {
    match sweep_days {
        Some(days) => run_sweep(days),
        None => run_cargo_clean(),
    }
}

fn run_cargo_clean() {
    println!("Cleaning build artifacts...");

    let output = match Command::new("cargo").arg("clean").output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Failed to run cargo clean: {e}");
            return;
        }
    };

    if output.status.success() {
        println!("Build artifacts removed.");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("cargo clean failed: {stderr}");
    }
}

fn run_sweep(days: u32) {
    if !is_sweep_installed() {
        eprintln!("cargo-sweep is required for time-based cleanup.");
        eprintln!("Install it with: cargo install cargo-sweep");
        return;
    }

    println!("Cleaning build artifacts older than {days} days...");

    let output = match Command::new("cargo")
        .args(["sweep", "--time", &days.to_string()])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Failed to run cargo sweep: {e}");
            return;
        }
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(cleaned) = parse_sweep_output(&stdout) {
            println!("{cleaned}");
        } else {
            println!("Cleanup complete.");
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Sweep failed: {stderr}");
    }
}

/// Run sweep silently during serve startup, return cleaned size if any
pub fn run_silent(days: u32) -> Option<String> {
    if !is_sweep_installed() {
        return None;
    }

    let output = Command::new("cargo")
        .args(["sweep", "--time", &days.to_string()])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_sweep_output(&stdout)
    } else {
        None
    }
}

fn is_sweep_installed() -> bool {
    Command::new("cargo")
        .args(["sweep", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn parse_sweep_output(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("Cleaned")
            && (line.contains("GiB") || line.contains("MiB") || line.contains("KiB"))
        {
            if let Some(start) = line.find("Cleaned") {
                let size_part = &line[start..];
                return Some(size_part.to_string());
            }
        }
    }
    None
}

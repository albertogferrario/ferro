use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use criterion::Criterion;

fn ferro_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ferro"))
}

#[test]
#[ignore = "wall-clock benchmark; run with FERRO_BENCH=1 (builds a full project in tmpdir)"]
fn benchmark_new_project() {
    if std::env::var("FERRO_BENCH").is_err() {
        eprintln!("skipping: set FERRO_BENCH=1 to run benchmark");
        return;
    }

    let mut c = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(600));

    c.bench_function("new_project_to_cargo_build", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;

            for _ in 0..iters {
                let tmp = tempfile::tempdir().expect("tempdir");
                let project_dir = tmp.path().join("bench-app");

                // Step 1: ferro new (CWD = parent; new.rs creates a bench-app/ subdir)
                let t = Instant::now();
                let status = Command::new(ferro_bin())
                    .args(["new", "bench-app", "--no-interaction", "--no-git"])
                    .current_dir(tmp.path())
                    .status()
                    .expect("ferro new failed to spawn");
                let code = status.code();
                assert!(status.success(), "ferro new exited non-zero: {code:?}");
                let step1 = t.elapsed();

                // Step 2: ferro make:auth (CWD = project root)
                let t = Instant::now();
                let status = Command::new(ferro_bin())
                    .args(["make:auth"])
                    .current_dir(&project_dir)
                    .status()
                    .expect("ferro make:auth failed to spawn");
                let code = status.code();
                assert!(
                    status.success(),
                    "ferro make:auth exited non-zero: {code:?}"
                );
                let step2 = t.elapsed();

                // Step 3a: ferro make:scaffold Article
                let t = Instant::now();
                let status = Command::new(ferro_bin())
                    // Flags MUST precede the positional fields: `[FIELDS]...` is a greedy
                    // trailing positional, so `--flags` placed after the fields are parsed
                    // as field names ("Invalid field name: '--no-smart-defaults'"). Usage:
                    // `make:scaffold [OPTIONS] <NAME> [FIELDS]...`. Discovered via the cold run.
                    .args([
                        "make:scaffold",
                        "--no-smart-defaults",
                        "-q",
                        "-y",
                        "--api",
                        "Article",
                        "title:string",
                        "body:text",
                    ])
                    .current_dir(&project_dir)
                    .status()
                    .expect("ferro make:scaffold Article failed to spawn");
                let code = status.code();
                assert!(
                    status.success(),
                    "ferro make:scaffold Article exited non-zero: {code:?}"
                );
                let step3a = t.elapsed();

                // Step 3b: ferro make:scaffold Product
                let t = Instant::now();
                let status = Command::new(ferro_bin())
                    .args([
                        "make:scaffold",
                        "--no-smart-defaults",
                        "-q",
                        "-y",
                        "--api",
                        "Product",
                        "name:string",
                        "price:float",
                    ])
                    .current_dir(&project_dir)
                    .status()
                    .expect("ferro make:scaffold Product failed to spawn");
                let code = status.code();
                assert!(
                    status.success(),
                    "ferro make:scaffold Product exited non-zero: {code:?}"
                );
                let step3b = t.elapsed();

                // Step 3c: ferro make:scaffold Order
                let t = Instant::now();
                let status = Command::new(ferro_bin())
                    .args([
                        "make:scaffold",
                        "--no-smart-defaults",
                        "-q",
                        "-y",
                        "--api",
                        "Order",
                        "status:string",
                        "total:float",
                    ])
                    .current_dir(&project_dir)
                    .status()
                    .expect("ferro make:scaffold Order failed to spawn");
                let code = status.code();
                assert!(
                    status.success(),
                    "ferro make:scaffold Order exited non-zero: {code:?}"
                );
                let step3c = t.elapsed();

                // Step 4: ferro make:job EmailNotification
                let t = Instant::now();
                let status = Command::new(ferro_bin())
                    .args(["make:job", "EmailNotification"])
                    .current_dir(&project_dir)
                    .status()
                    .expect("ferro make:job failed to spawn");
                let code = status.code();
                assert!(status.success(), "ferro make:job exited non-zero: {code:?}");
                let step4 = t.elapsed();

                // Step 5: cargo build — asserts exit 0 (SC#2)
                let t = Instant::now();
                let status = Command::new("cargo")
                    .args(["build"])
                    .current_dir(&project_dir)
                    .status()
                    .expect("cargo build failed to spawn");
                let code = status.code();
                assert!(status.success(), "cargo build exited non-zero: {code:?}");
                let step5 = t.elapsed();

                println!("=== COMP-04 Benchmark Results ===");
                println!("Step 1  ferro new:              {step1:?}");
                println!("Step 2  ferro make:auth:        {step2:?}");
                println!("Step 3a ferro make:scaffold A:  {step3a:?}");
                println!("Step 3b ferro make:scaffold P:  {step3b:?}");
                println!("Step 3c ferro make:scaffold O:  {step3c:?}");
                println!("Step 4  ferro make:job:         {step4:?}");
                println!("Step 5  cargo build:            {step5:?}");

                total += step1 + step2 + step3a + step3b + step3c + step4 + step5;

                // tmp drops here, cleaning up the tmpdir for each iteration
            }

            println!(
                "Total (avg over {iters} iters):  {:?}",
                total / iters as u32
            );

            total
        })
    });
    // criterion flushes output when c drops; final_summary() is not public API in 0.8.2
}

//! The `storage:link` command creates a symbolic link from public/storage to storage/app/public.

use std::fs;
use std::path::Path;

/// Run the storage:link command.
pub fn run(relative: bool) {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    // Define source and target paths
    let storage_path = current_dir.join("storage").join("app").join("public");
    let public_path = current_dir.join("public").join("storage");

    // Ensure source directory exists
    if !storage_path.exists() {
        println!("Creating storage/app/public directory...");
        if let Err(e) = fs::create_dir_all(&storage_path) {
            eprintln!("❌ Failed to create storage directory: {e}");
            std::process::exit(1);
        }
    }

    // Check if link already exists
    if public_path.exists() {
        if public_path.is_symlink() {
            println!("✓ Symbolic link already exists at public/storage");
            return;
        } else {
            eprintln!("❌ public/storage already exists and is not a symbolic link");
            eprintln!("   Remove it manually before running this command");
            std::process::exit(1);
        }
    }

    // Ensure public directory exists
    let public_dir = current_dir.join("public");
    if !public_dir.exists() {
        println!("Creating public directory...");
        if let Err(e) = fs::create_dir_all(&public_dir) {
            eprintln!("❌ Failed to create public directory: {e}");
            std::process::exit(1);
        }
    }

    // Create the symbolic link
    let target = if relative {
        // Relative path from public/storage to storage/app/public
        Path::new("..").join("storage").join("app").join("public")
    } else {
        storage_path.clone()
    };

    #[cfg(unix)]
    {
        if let Err(e) = std::os::unix::fs::symlink(&target, &public_path) {
            eprintln!("❌ Failed to create symbolic link: {e}");
            std::process::exit(1);
        }
    }

    #[cfg(windows)]
    {
        // On Windows, creating symlinks requires admin privileges or developer mode
        if let Err(e) = std::os::windows::fs::symlink_dir(&target, &public_path) {
            eprintln!("❌ Failed to create symbolic link: {}", e);
            eprintln!("   On Windows, you may need to run as Administrator");
            eprintln!("   or enable Developer Mode in Windows Settings");
            std::process::exit(1);
        }
    }

    println!("✓ Created symbolic link: public/storage -> storage/app/public");
    println!();
    println!("Your application can now serve files from the storage directory.");
    println!("Store files in storage/app/public and access them via /storage URL prefix.");
}

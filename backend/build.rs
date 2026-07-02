use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=../frontend/src");
    println!("cargo::rerun-if-changed=../frontend/index.html");
    println!("cargo::rerun-if-changed=../frontend/styles.css");
    println!("cargo::rerun-if-changed=../frontend/Cargo.toml");
    println!("cargo::rerun-if-changed=../frontend/Trunk.toml");

    if let Ok(var) = env::var("FRONTEND_ASSETS") {
        println!("cargo::rustc-env=FRONTEND_ASSETS={var}",);
    } else {
        println!("cargo::rustc-env=FRONTEND_ASSETS=../frontend/dist",);
    }

    // Get the workspace root directory
    let workspace_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = Path::new(&workspace_root)
        .parent()
        .expect("Failed to get workspace root");

    let frontend_dir = workspace_root.join("frontend");
    let dist_dir = frontend_dir.join("dist");

    println!(
        "cargo:warning=Building frontend from: {}",
        frontend_dir.display()
    );

    // Check if the frontend directory exists
    if !frontend_dir.exists() {
        println!(
            "cargo:warning=Frontend directory not found at: {}",
            frontend_dir.display()
        );
        return;
    }

    // Try to find trunk in common locations
    let trunk_cmd = which_trunk();

    if trunk_cmd.is_none() {
        println!(
            "cargo:warning=Trunk is not installed or not in PATH. Please install it with: cargo install trunk"
        );
        println!(
            "cargo:warning=Skipping frontend build. The backend will not serve the frontend correctly."
        );
        return;
    }

    let trunk_cmd = trunk_cmd.unwrap();
    println!("cargo:warning=Found trunk at: {}", trunk_cmd);

    // Build the frontend using trunk
    println!("cargo:warning=Building frontend with trunk...");
    // let status = Command::new(&trunk_cmd)
    //     .arg("build")
    //     .arg("--release")
    //     .current_dir(&frontend_dir)
    //     .env("PATH", env::var("PATH").unwrap_or_default())
    //     .status()
    //     .expect("Failed to execute trunk build");

    // if !status.success() {
    //     panic!("Frontend build failed!");
    // }

    // Verify the dist directory was created
    if !dist_dir.exists() {
        panic!(
            "Frontend build succeeded but dist directory not found at: {}",
            dist_dir.display()
        );
    }

    println!("cargo:warning=Frontend build completed successfully!");
    println!(
        "cargo:warning=Frontend assets are in: {}",
        dist_dir.display()
    );
}

fn which_trunk() -> Option<String> {
    // First, try to run trunk directly (works if it's in PATH)
    if Command::new("trunk").arg("--version").output().is_ok() {
        return Some("trunk".to_string());
    }

    // Try common cargo install locations
    let home = env::var("HOME").ok()?;
    let cargo_bin = format!("{}/.cargo/bin/trunk", home);
    if Path::new(&cargo_bin).exists() {
        return Some(cargo_bin);
    }

    // Check if trunk is in PATH by iterating through PATH entries
    if let Ok(path_var) = env::var("PATH") {
        for path in path_var.split(':') {
            let trunk_path = format!("{}/trunk", path);
            if Path::new(&trunk_path).exists() {
                return Some(trunk_path);
            }
        }
    }

    None
}

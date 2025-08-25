#![allow(unused)]

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    gst_plugin_version_helper::info();

    // Only run this when ort feature is enabled and we're on Windows
    #[cfg(all(feature = "ort", target_os = "windows"))]
    copy_onnx_dlls();
    
    // Also set up a rerun trigger for when ort completes
    println!("cargo:rerun-if-env-changed=ORT_STRATEGY");
}

#[cfg(all(feature = "ort", target_os = "windows"))]
fn copy_onnx_dlls() {
    println!("cargo:rerun-if-changed=build.rs");
    
    let target_dir = get_target_dir();
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let profile_dir = target_dir.join(&profile);
    
    // ONNX Runtime DLL files to copy
    let dll_files = ["onnxruntime.dll", "onnxruntime_providers_shared.dll"];
    
    // Destination directories
    let deps_dir = profile_dir.join("deps");
    let examples_dir = profile_dir.join("examples");
    
    // Create directories if they don't exist
    let _ = fs::create_dir_all(&deps_dir);
    let _ = fs::create_dir_all(&examples_dir);
    
    for dll_name in &dll_files {
        let source = profile_dir.join(dll_name);
        
        if source.exists() {
            // Copy to deps directory (for tests)
            let deps_dest = deps_dir.join(dll_name);
            if let Err(e) = fs::copy(&source, &deps_dest) {
                println!("cargo:warning=Failed to copy {dll_name} to deps: {e}");
            } else {
                println!("cargo:warning=Copied {dll_name} to deps directory for tests");
            }
            
            // Copy to examples directory (for examples)
            let examples_dest = examples_dir.join(dll_name);
            if let Err(e) = fs::copy(&source, &examples_dest) {
                println!("cargo:warning=Failed to copy {dll_name} to examples: {e}");
            } else {
                println!("cargo:warning=Copied {dll_name} to examples directory");
            }
        } else {
            println!("cargo:warning=ONNX Runtime DLL not found: {}. This may cause runtime errors with ort feature.", source.display());
            println!("cargo:warning=The DLLs should be automatically downloaded by the ort crate build process.");
        }
    }
}

#[cfg(all(feature = "ort", target_os = "windows"))]
fn get_target_dir() -> PathBuf {
    // Try to get target directory from environment variables
    if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
        return PathBuf::from(target_dir);
    }
    
    // Fallback: try to find target directory relative to manifest dir
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = PathBuf::from(manifest_dir);
    
    // Look for target directory in workspace root or current directory
    let mut current = manifest_path.as_path();
    loop {
        let target_candidate = current.join("target");
        if target_candidate.exists() && target_candidate.is_dir() {
            return target_candidate;
        }
        
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }
    
    // Final fallback
    PathBuf::from("target")
}

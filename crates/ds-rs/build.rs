#![allow(unused)]
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Only run this when ort feature is enabled and we're on Windows
    #[cfg(all(feature = "ort", target_os = "windows"))]
    {
        let result = copy_onnx_dlls();
        if let Err(e) = result {
            println!("cargo:warning=DLL setup issue: {}", e);
        }
    }
    
    // Also set up a rerun trigger for when ort completes
    println!("cargo:rerun-if-env-changed=ORT_STRATEGY");
    println!("cargo:rerun-if-env-changed=ORT_DYLIB_PATH");
    println!("cargo:rerun-if-env-changed=ORT_LIB_LOCATION");
}

#[cfg(all(feature = "ort", target_os = "windows"))]
fn copy_onnx_dlls() -> Result<(), String> {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Check environment variables for custom DLL paths
    if let Ok(dylib_path) = env::var("ORT_DYLIB_PATH") {
        println!("cargo:warning=Using ORT_DYLIB_PATH: {}", dylib_path);
        return Ok(());
    }
    
    let target_dir = get_target_dir();
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let profile_dir = target_dir.join(&profile);
    
    println!("cargo:warning=Looking for ONNX Runtime DLLs in: {}", profile_dir.display());
    
    // ONNX Runtime DLL files to copy
    let dll_files = ["onnxruntime.dll", "onnxruntime_providers_shared.dll"];
    
    // First, try to find the DLLs in various locations
    let mut dll_sources = Vec::new();
    for dll_name in &dll_files {
        let source = find_dll(&profile_dir, dll_name)?;
        dll_sources.push((dll_name, source));
    }
    
    // Destination directories
    let destinations = vec![
        profile_dir.join("deps"),
        profile_dir.join("examples"),
        profile_dir.clone(), // Also copy to main target directory
    ];
    
    // Create directories if they don't exist
    for dest_dir in &destinations {
        fs::create_dir_all(dest_dir).map_err(|e| {
            format!("Failed to create directory {}: {}", dest_dir.display(), e)
        })?;
    }
    
    // Copy DLLs to all necessary locations
    for (dll_name, source) in &dll_sources {
        println!("cargo:warning=Found {} at: {}", dll_name, source.display());
        
        // Verify the DLL is valid (basic size check)
        let metadata = fs::metadata(&source).map_err(|e| {
            format!("Failed to read metadata for {}: {}", source.display(), e)
        })?;
        
        if metadata.len() < 1024 {
            return Err(format!(
                "DLL {} seems too small ({} bytes), might be corrupted",
                dll_name, metadata.len()
            ));
        }
        
        for dest_dir in &destinations {
            let dest = dest_dir.join(dll_name);
            
            // Skip if already exists and has same size
            if dest.exists() {
                if let Ok(dest_meta) = fs::metadata(&dest) {
                    if dest_meta.len() == metadata.len() {
                        println!("cargo:warning={} already up-to-date in {}", dll_name, dest_dir.file_name().unwrap_or_default().to_string_lossy());
                        continue;
                    }
                }
            }
            
            fs::copy(&source, &dest).map_err(|e| {
                format!("Failed to copy {} to {}: {}", dll_name, dest.display(), e)
            })?;
            
            println!("cargo:warning=Successfully copied {} to {}", dll_name, dest_dir.file_name().unwrap_or_default().to_string_lossy());
        }
    }
    
    println!("cargo:warning=ONNX Runtime DLL setup completed successfully");
    println!("cargo:warning=If you still get 0xc000007b errors, ensure Visual C++ Redistributables are installed");
    
    Ok(())
}

#[cfg(all(feature = "ort", target_os = "windows"))]
fn find_dll(profile_dir: &Path, dll_name: &str) -> Result<PathBuf, String> {
    // List of potential locations to search for the DLL
    let search_paths = vec![
        // Direct path in profile directory
        profile_dir.join(dll_name),
        // ORT might download to a subdirectory
        profile_dir.join("onnxruntime").join(dll_name),
        // Check parent directories (workspace root)
        profile_dir.parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("target").join(dll_name))
            .unwrap_or_default(),
        // Check if ORT_LIB_LOCATION is set
        env::var("ORT_LIB_LOCATION")
            .ok()
            .map(PathBuf::from)
            .map(|p| p.join(dll_name))
            .unwrap_or_default(),
    ];
    
    for path in search_paths {
        if path.exists() && path.is_file() {
            return Ok(path);
        }
    }
    
    // If not found, provide helpful error message
    Err(format!(
        "Could not find {}. The ort crate should download it automatically. \
         You can also set ORT_DYLIB_PATH environment variable to point to the DLL location, \
         or download ONNX Runtime manually from https://github.com/microsoft/onnxruntime/releases",
        dll_name
    ))
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

//! Windows DLL validation and loading helper
//!
//! This module provides utilities for validating and diagnosing DLL loading issues
//! on Windows, particularly for ONNX Runtime dependencies.

use std::env;
use std::path::Path;

/// Errors that can occur during DLL validation
#[derive(Debug, thiserror::Error)]
pub enum DllError {
    #[error("DLL not found: {path}")]
    NotFound { path: String },

    #[error("DLL architecture mismatch: expected {expected}, got {actual}")]
    ArchitectureMismatch { expected: String, actual: String },

    #[error("Missing Visual C++ Redistributable: {details}")]
    MissingRedistributable { details: String },

    #[error("DLL loading failed: {details}")]
    LoadingFailed { details: String },

    #[error("Environment variable {var} points to invalid location: {path}")]
    InvalidEnvironmentPath { var: String, path: String },
}

/// Information about DLL validation results
#[derive(Debug)]
pub struct DllValidationInfo {
    pub dll_name: String,
    pub found_at: Option<String>,
    pub is_valid: bool,
    pub error: Option<DllError>,
    pub suggestions: Vec<String>,
}

/// Validates ONNX Runtime DLLs are properly set up on Windows
#[cfg(target_os = "windows")]
pub fn validate_onnx_runtime_dlls() -> Vec<DllValidationInfo> {
    let mut results = Vec::new();

    // Check for ORT_DYLIB_PATH environment variable
    if let Ok(dylib_path) = env::var("ORT_DYLIB_PATH") {
        log::info!("ORT_DYLIB_PATH is set to: {}", dylib_path);
        if !Path::new(&dylib_path).exists() {
            results.push(DllValidationInfo {
                dll_name: "Environment".to_string(),
                found_at: Some(dylib_path.clone()),
                is_valid: false,
                error: Some(DllError::InvalidEnvironmentPath {
                    var: "ORT_DYLIB_PATH".to_string(),
                    path: dylib_path,
                }),
                suggestions: vec![
                    "Check that ORT_DYLIB_PATH points to a valid DLL file".to_string(),
                    "Or unset ORT_DYLIB_PATH to use automatic DLL discovery".to_string(),
                ],
            });
            return results;
        }
    }

    // DLLs to check
    let required_dlls = ["onnxruntime.dll", "onnxruntime_providers_shared.dll"];

    // Get the directory where the executable is located
    let exe_path = env::current_exe().ok();
    let exe_dir = exe_path.as_ref().and_then(|p| p.parent());

    for dll_name in &required_dlls {
        let mut info = DllValidationInfo {
            dll_name: dll_name.to_string(),
            found_at: None,
            is_valid: false,
            error: None,
            suggestions: Vec::new(),
        };

        // Check if DLL exists next to executable
        if let Some(dir) = exe_dir {
            let dll_path = dir.join(dll_name);
            if dll_path.exists() {
                info.found_at = Some(dll_path.display().to_string());
                info.is_valid = validate_dll_file(&dll_path);
                if !info.is_valid {
                    info.error = Some(DllError::LoadingFailed {
                        details: "DLL file exists but may be corrupted or wrong architecture"
                            .to_string(),
                    });
                    info.suggestions
                        .push("Ensure the DLL is for x64 architecture".to_string());
                    info.suggestions
                        .push("Try re-downloading ONNX Runtime".to_string());
                }
            } else {
                info.error = Some(DllError::NotFound {
                    path: dll_path.display().to_string(),
                });
                info.suggestions
                    .push(format!("Copy {} to: {}", dll_name, dll_path.display()));
                info.suggestions
                    .push("Or set ORT_DYLIB_PATH environment variable".to_string());
                info.suggestions
                    .push("Or rebuild with: cargo clean && cargo build --features ort".to_string());
            }
        }

        results.push(info);
    }

    // Check for Visual C++ Redistributables
    let vc_redist_info = check_vc_redistributables();
    if !vc_redist_info.is_valid {
        results.push(vc_redist_info);
    }

    results
}

/// Checks if Visual C++ Redistributables are installed
#[cfg(target_os = "windows")]
fn check_vc_redistributables() -> DllValidationInfo {
    let mut info = DllValidationInfo {
        dll_name: "Visual C++ Redistributables".to_string(),
        found_at: None,
        is_valid: true,
        error: None,
        suggestions: Vec::new(),
    };

    // Check for common VC++ runtime DLLs
    let vc_dlls = ["MSVCP140.dll", "VCRUNTIME140.dll"];
    let system32 = Path::new("C:\\Windows\\System32");

    for dll in &vc_dlls {
        let dll_path = system32.join(dll);
        if !dll_path.exists() {
            info.is_valid = false;
            info.error = Some(DllError::MissingRedistributable {
                details: format!("{} not found in System32", dll),
            });
            info.suggestions
                .push("Download and install Visual C++ Redistributable from:".to_string());
            info.suggestions
                .push("https://aka.ms/vs/17/release/vc_redist.x64.exe".to_string());
            break;
        }
    }

    info
}

/// Basic validation of a DLL file
#[cfg(target_os = "windows")]
fn validate_dll_file(path: &Path) -> bool {
    // Basic checks: file exists and has reasonable size
    if let Ok(metadata) = std::fs::metadata(path) {
        // DLLs should be at least a few KB
        metadata.len() > 1024
    } else {
        false
    }
}

/// Stub implementation for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn validate_onnx_runtime_dlls() -> Vec<DllValidationInfo> {
    Vec::new()
}

/// Prints a diagnostic report for DLL issues
pub fn print_dll_diagnostic_report() {
    #[cfg(target_os = "windows")]
    {
        println!("\n=== ONNX Runtime DLL Diagnostic Report ===\n");

        let validations = validate_onnx_runtime_dlls();
        let all_valid = validations.iter().all(|v| v.is_valid);

        if all_valid {
            println!(" All DLLs are properly configured!");
        } else {
            println!(" DLL configuration issues detected:\n");

            for validation in validations {
                if !validation.is_valid {
                    println!(
                        "  {} {}",
                        if validation.is_valid { "OK" } else { "BAD" },
                        validation.dll_name
                    );

                    if let Some(found_at) = &validation.found_at {
                        println!("    Location: {}", found_at);
                    }

                    if let Some(error) = &validation.error {
                        println!("    Error: {}", error);
                    }

                    if !validation.suggestions.is_empty() {
                        println!("    Suggestions:");
                        for suggestion in &validation.suggestions {
                            println!("      - {}", suggestion);
                        }
                    }
                    println!();
                }
            }

            println!("\n=== Quick Fix Commands ===\n");
            println!("1. Clean and rebuild:");
            println!("   cargo clean");
            println!("   cargo build --features ort");
            println!();
            println!("2. Set custom DLL path (if you have ONNX Runtime installed elsewhere):");
            println!("   set ORT_DYLIB_PATH=C:\\path\\to\\onnxruntime.dll");
            println!();
            println!("3. Download ONNX Runtime manually:");
            println!("   Visit: https://github.com/microsoft/onnxruntime/releases");
            println!("   Download: onnxruntime-win-x64-*.zip");
            println!("   Extract DLLs to your target\\debug\\examples\\ directory");
        }

        println!("\n=========================================\n");
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!("DLL validation is only relevant on Windows platforms.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dll_validation_structure() {
        let info = DllValidationInfo {
            dll_name: "test.dll".to_string(),
            found_at: Some("/path/to/test.dll".to_string()),
            is_valid: true,
            error: None,
            suggestions: vec![],
        };

        assert_eq!(info.dll_name, "test.dll");
        assert!(info.is_valid);
        assert!(info.error.is_none());
        assert!(info.suggestions.is_empty());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_vc_redistributable_check() {
        // This should pass on most Windows development machines
        let info = check_vc_redistributables();
        // We won't assert it's valid since it depends on the system
        assert_eq!(info.dll_name, "Visual C++ Redistributables");
    }
}

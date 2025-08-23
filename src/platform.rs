use crate::error::Result;
use std::env;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Jetson,
    X86,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub platform: Platform,
    pub cuda_version: Option<String>,
    pub gpu_id: Option<u32>,
    pub compute_capability: Option<String>,
}

impl PlatformInfo {
    pub fn detect() -> Result<Self> {
        let platform = detect_platform();
        let cuda_version = detect_cuda_version();
        let gpu_id = detect_gpu_id();
        let compute_capability = detect_compute_capability();
        
        Ok(PlatformInfo {
            platform,
            cuda_version,
            gpu_id,
            compute_capability,
        })
    }
    
    pub fn is_jetson(&self) -> bool {
        matches!(self.platform, Platform::Jetson)
    }
    
    pub fn is_x86(&self) -> bool {
        matches!(self.platform, Platform::X86)
    }
    
    pub fn has_nvidia_hardware(&self) -> bool {
        self.cuda_version.is_some() || 
        Path::new("/usr/local/cuda").exists() ||
        Path::new("/opt/nvidia/deepstream").exists()
    }
    
    pub fn get_gpu_id(&self) -> u32 {
        self.gpu_id.unwrap_or(0)
    }
    
    pub fn get_batch_timeout(&self) -> u32 {
        match self.platform {
            Platform::Jetson => 40000,  // 40ms for Jetson
            _ => 4000,                  // 4ms for x86
        }
    }
    
    pub fn get_compute_mode(&self) -> i32 {
        match self.platform {
            Platform::Jetson => 0,  // GPU_DEVICE
            _ => 1,                // GPU_DEVICE_ID
        }
    }
}

fn detect_platform() -> Platform {
    // Check for Jetson-specific files
    if Path::new("/etc/nv_tegra_release").exists() ||
       Path::new("/sys/module/tegra_fuse/parameters/tegra_chip_id").exists() {
        return Platform::Jetson;
    }
    
    // Check architecture
    let arch = env::consts::ARCH;
    if arch == "x86_64" || arch == "x86" {
        return Platform::X86;
    }
    
    // Check CUDA version as fallback
    if let Some(cuda_ver) = detect_cuda_version() {
        if cuda_ver.starts_with("10.2") {
            return Platform::Jetson;
        } else if cuda_ver.starts_with("11.") || cuda_ver.starts_with("12.") {
            return Platform::X86;
        }
    }
    
    Platform::Unknown
}

fn detect_cuda_version() -> Option<String> {
    // First check environment variable
    if let Ok(cuda_ver) = env::var("CUDA_VER") {
        return Some(cuda_ver);
    }
    
    // Check nvcc version
    if let Ok(output) = std::process::Command::new("nvcc")
        .arg("--version")
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        // Parse version from nvcc output
        for line in output_str.lines() {
            if line.contains("release") {
                if let Some(version) = line.split("release").nth(1) {
                    if let Some(ver) = version.split(',').next() {
                        return Some(ver.trim().to_string());
                    }
                }
            }
        }
    }
    
    // Check for CUDA installation directories
    if Path::new("/usr/local/cuda-12.0").exists() {
        return Some("12.0".to_string());
    }
    if Path::new("/usr/local/cuda-11.4").exists() {
        return Some("11.4".to_string());
    }
    if Path::new("/usr/local/cuda-10.2").exists() {
        return Some("10.2".to_string());
    }
    
    None
}

fn detect_gpu_id() -> Option<u32> {
    // Check environment variable
    if let Ok(gpu_id) = env::var("GPU_ID") {
        if let Ok(id) = gpu_id.parse::<u32>() {
            return Some(id);
        }
    }
    
    // Default to GPU 0 if NVIDIA hardware is detected
    if Path::new("/dev/nvidia0").exists() {
        return Some(0);
    }
    
    None
}

fn detect_compute_capability() -> Option<String> {
    // This would require nvidia-smi or CUDA API calls
    // For now, return common capabilities based on platform
    let platform = detect_platform();
    match platform {
        Platform::Jetson => Some("7.2".to_string()), // Common for Xavier
        Platform::X86 => Some("7.5".to_string()),    // Common for RTX series
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_detection() {
        let info = PlatformInfo::detect().unwrap();
        println!("Detected platform: {:?}", info);
        
        // Platform should be detected
        assert!(info.platform != Platform::Unknown || !info.has_nvidia_hardware());
    }
    
    #[test]
    fn test_platform_properties() {
        let info = PlatformInfo::detect().unwrap();
        
        // Test that methods don't panic
        let _ = info.is_jetson();
        let _ = info.is_x86();
        let _ = info.has_nvidia_hardware();
        let _ = info.get_gpu_id();
        let _ = info.get_batch_timeout();
        let _ = info.get_compute_mode();
    }
}
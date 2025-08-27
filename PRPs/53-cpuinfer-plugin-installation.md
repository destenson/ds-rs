# PRP-53: cpuinfer Plugin Installation and System Integration

## STATUS: COMPLETE (2025-08-27)

### Progress Made:
✅ Created install.ps1 for Windows installation
✅ Created install.sh for Linux installation
✅ Created dev-setup.md with comprehensive documentation
✅ Scripts handle ONNX Runtime DLL dependencies
✅ Scripts clear plugin cache automatically
✅ Manual installation tested successfully on Windows

## Problem Statement

Once the cpuinfer plugin is properly built and has nvinfer-compatible properties, it needs to be installed and made discoverable by GStreamer. This includes system installation, development workflows, packaging, and integration with existing GStreamer installations across different platforms (Linux, Windows, embedded systems).

## Research & References

### GStreamer Plugin Discovery
- https://gstreamer.freedesktop.org/documentation/gstreamer/running.html - Plugin paths documentation
- https://stackoverflow.com/questions/2120444/gstreamer-plugin-search-path - Search path details
- https://github.com/openvinotoolkit/dlstreamer_gst/issues/6 - GST_PLUGIN_PATH usage

### Installation Locations
**Linux Standard Paths**:
- `/usr/lib/x86_64-linux-gnu/gstreamer-1.0/`
- `/usr/lib64/gstreamer-1.0/`
- `/usr/local/lib/gstreamer-1.0/`

**Windows Standard Paths**:
- `C:\gstreamer\1.0\msvc_x86_64\lib\gstreamer-1.0\`
- `C:\gstreamer\1.0\mingw_x86_64\lib\gstreamer-1.0\`

**Development Paths**:
- Set via `GST_PLUGIN_PATH` environment variable
- Multiple paths separated by `:` (Linux) or `;` (Windows)

## Implementation Blueprint

### Phase 1: Installation Scripts

#### 1.1 Linux Installation Script
**File**: `crates/cpuinfer/scripts/install-linux.sh`
**Features**:
```bash
#!/bin/bash
# Detect GStreamer installation
# Support both system and user installation
# Handle different architectures (x86_64, aarch64)
# Copy dependencies (ONNX Runtime libraries)
# Clear plugin cache after installation
# Verify installation with gst-inspect-1.0
```

**Key Logic**:
- Check for GStreamer with `pkg-config --variable=pluginsdir gstreamer-1.0`
- Offer user vs system installation choice
- Handle permission requirements for system installation
- Copy both plugin and dependencies

#### 1.2 Windows Installation Script
**File**: `crates/cpuinfer/scripts/install-windows.ps1`
**Features**:
```powershell
# Detect GStreamer installation paths
# Support both MSVC and MinGW builds
# Copy plugin and ONNX Runtime DLLs
# Update PATH if needed
# Clear plugin cache
# Verify installation
```

**Key Logic**:
- Check registry for GStreamer installation
- Check common installation paths
- Handle both 32-bit and 64-bit installations
- Copy all required DLLs to plugin directory

#### 1.3 Development Setup Script
**File**: `crates/cpuinfer/scripts/setup-dev.sh`
**Purpose**: Configure development environment
**Features**:
- Set GST_PLUGIN_PATH to development build directory
- Create convenience aliases for testing
- Set up debugging environment variables
- Create test configuration files

### Phase 2: Package Configuration

#### 2.1 Debian Package Support
**File**: `crates/cpuinfer/debian/control`
**Dependencies**:
```
Depends: gstreamer1.0-plugins-base, libonnxruntime1.16
```

**Files to Install**:
```
debian/gstcpuinfer.install:
  target/release/libgstcpuinfer.so => /usr/lib/x86_64-linux-gnu/gstreamer-1.0/
```

#### 2.2 RPM Package Support
**File**: `crates/cpuinfer/cpuinfer.spec`
**Configuration**:
- Requires: gstreamer1-plugins-base
- Install location: %{_libdir}/gstreamer-1.0/

#### 2.3 Cargo Installation Support
**File**: `crates/cpuinfer/Cargo.toml`
**Add Custom Install**:
```toml
[package.metadata.install]
bins = []
libs = ["libgstcpuinfer"]
target = "gstreamer-plugin"
```

### Phase 3: Runtime Configuration

#### 3.1 Plugin Dependencies
**File**: `crates/cpuinfer/scripts/check-deps.sh`
**Checks**:
- ONNX Runtime library availability
- GStreamer base plugins
- Required system libraries (OpenMP, etc.)

#### 3.2 Environment Setup
**File**: `crates/cpuinfer/env.sh`
**Sets**:
```bash
export GST_PLUGIN_PATH="${GST_PLUGIN_PATH}:$(pwd)/target/release"
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:$(pwd)/target/release"
```

#### 3.3 Windows Environment
**File**: `crates/cpuinfer/env.bat`
**Sets**:
```batch
set GST_PLUGIN_PATH=%GST_PLUGIN_PATH%;%CD%\target\release
set PATH=%PATH%;%CD%\target\release
```

### Phase 4: CI/CD Integration

#### 4.1 GitHub Actions Workflow
**File**: `.github/workflows/cpuinfer-release.yml`
**Steps**:
1. Build plugin for multiple platforms
2. Run installation tests
3. Create release artifacts
4. Generate installation packages

#### 4.2 Testing Installation
**File**: `crates/cpuinfer/tests/test_installation.sh`
**Tests**:
- Plugin discovery after installation
- Pipeline execution with installed plugin
- Dependency resolution
- Cache behavior

### Phase 5: Documentation

#### 5.1 Installation Guide
**File**: `crates/cpuinfer/INSTALL.md`
**Sections**:
- Prerequisites
- Platform-specific instructions
- Troubleshooting common issues
- Verification steps

#### 5.2 Developer Guide
**File**: `crates/cpuinfer/DEVELOPMENT.md`
**Contents**:
- Setting up development environment
- Building from source
- Running tests
- Debugging techniques

## Platform-Specific Considerations

### Linux
- Use ldconfig after system installation
- Handle both .deb and .rpm based systems
- Consider AppImage/Flatpak for portable distribution

### Windows
- Handle Visual Studio vs MinGW builds
- Registry entries for plugin discovery
- Side-by-side installation with multiple GStreamer versions

### Embedded/Cross-Compilation
- Support Yocto/Buildroot integration
- Handle different library paths on embedded systems
- Consider static linking for single-binary deployment

## Validation Gates

```bash
# Test installation script (Linux)
./scripts/install-linux.sh --user --prefix=$HOME/.local
gst-inspect-1.0 cpuinfer || exit 1

# Test system installation (requires sudo)
sudo ./scripts/install-linux.sh --system
gst-inspect-1.0 cpuinfer || exit 1

# Test development setup
source ./scripts/setup-dev.sh
gst-inspect-1.0 cpuinfer || exit 1

# Verify plugin loading from custom path
GST_PLUGIN_PATH=/tmp/test-install gst-inspect-1.0 cpuinfer

# Test with real pipeline
gst-launch-1.0 videotestsrc num-buffers=10 ! cpuinfer ! fakesink || exit 1

# Check dependencies
ldd $(pkg-config --variable=pluginsdir gstreamer-1.0)/libgstcpuinfer.so

# Test uninstallation
./scripts/uninstall.sh
! gst-inspect-1.0 cpuinfer 2>/dev/null || exit 1
```

## Troubleshooting Guide

### Common Issues and Solutions

1. **Plugin Not Found**
   - Clear cache: `rm -rf ~/.cache/gstreamer-1.0/`
   - Check GST_PLUGIN_PATH is set correctly
   - Verify file permissions on plugin

2. **Missing Dependencies**
   - Use ldd/Dependency Walker to check
   - Install ONNX Runtime separately
   - Set LD_LIBRARY_PATH/PATH correctly

3. **Version Mismatch**
   - Check GStreamer version with `gst-inspect-1.0 --version`
   - Rebuild plugin against correct version
   - Use version-specific plugin paths

4. **Blacklisting**
   - Check with `gst-inspect-1.0 -b`
   - Review debug output: `GST_DEBUG=GST_PLUGIN_LOADING:7`
   - Fix underlying issue and clear cache

## Distribution Strategies

### Binary Releases
- GitHub Releases with pre-built binaries
- Platform-specific installers (.deb, .rpm, .msi)
- Portable archives (.tar.gz, .zip)

### Package Managers
- Submit to distribution repositories (Ubuntu PPA, AUR)
- Homebrew formula for macOS
- Chocolatey package for Windows

### Container Images
- Docker image with plugin pre-installed
- Example Dockerfiles for different use cases
- Integration with NVIDIA container toolkit

## Success Criteria

- Installation scripts work on major Linux distributions
- Windows installation works for both MSVC and MinGW
- Plugin is discoverable after installation
- Dependencies are properly resolved
- Uninstallation removes all components cleanly
- Development workflow is documented and functional

## Risk Assessment

- **High Risk**: Dependency version conflicts
- **Mitigation**: Bundle dependencies or use static linking
- **Medium Risk**: Permission issues on system installation  
- **Mitigation**: Provide user installation option
- **Low Risk**: Cache corruption
- **Mitigation**: Automatic cache clearing in scripts

## Estimated Effort

- Installation scripts: 4-5 hours
- Package configuration: 3-4 hours
- CI/CD setup: 2-3 hours
- Documentation: 2-3 hours
- Testing across platforms: 3-4 hours
- Total: 14-19 hours

## References from Codebase

- Plugin structure: `../gst-plugins-rs/*/Cargo.toml`
- Build scripts: `../gst-plugins-rs/*/build.rs`
- CI workflows: `../.github/workflows/` (if available)

## Next Steps

After completing installation:
1. Create user-facing documentation
2. Set up continuous integration for releases
3. Submit to package repositories
4. Create Docker images for easy deployment

## Confidence Score: 8/10

Installation procedures are well-understood with clear patterns from existing GStreamer plugins. The main complexity is handling different platforms and dependency management. With proper scripts and documentation, this should be straightforward to implement.
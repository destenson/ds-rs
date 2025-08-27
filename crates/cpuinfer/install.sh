#!/bin/bash
# Installation script for cpuinfer GStreamer plugin on Linux

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
BUILD_TYPE="release"
INSTALL_MODE="user"
DRY_RUN=false
VERBOSE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_TYPE="debug"
            shift
            ;;
        --system)
            INSTALL_MODE="system"
            shift
            ;;
        --user)
            INSTALL_MODE="user"
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --debug        Install debug build (default: release)"
            echo "  --system       Install system-wide (requires sudo)"
            echo "  --user         Install for current user (default)"
            echo "  --dry-run      Show what would be done without doing it"
            echo "  --verbose      Show verbose output"
            echo "  --help         Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Find GStreamer plugin directory
find_gstreamer_plugin_dir() {
    if [ "$INSTALL_MODE" = "system" ]; then
        # Try to find system plugin directory
        if command -v pkg-config &> /dev/null; then
            PLUGIN_DIR=$(pkg-config --variable=pluginsdir gstreamer-1.0 2>/dev/null || true)
            if [ -n "$PLUGIN_DIR" ]; then
                echo "$PLUGIN_DIR"
                return
            fi
        fi
        
        # Common system paths
        for path in /usr/lib/x86_64-linux-gnu/gstreamer-1.0 \
                   /usr/lib64/gstreamer-1.0 \
                   /usr/lib/gstreamer-1.0 \
                   /usr/local/lib/gstreamer-1.0; do
            if [ -d "$path" ]; then
                echo "$path"
                return
            fi
        done
    else
        # User installation
        echo "$HOME/.local/share/gstreamer-1.0/plugins"
    fi
}

# Main installation
info "cpuinfer GStreamer Plugin Installer for Linux"
info "=============================================="

# Find project root and plugin file
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
PLUGIN_FILE="$PROJECT_ROOT/target/$BUILD_TYPE/libgstcpuinfer.so"

if [ ! -f "$PLUGIN_FILE" ]; then
    error "Plugin not found at: $PLUGIN_FILE"
    info "Please build the plugin first with: cargo build --release -p cpuinfer"
    exit 1
fi

info "Found plugin: $PLUGIN_FILE"

# Find or create plugin directory
PLUGIN_DIR=$(find_gstreamer_plugin_dir)
if [ -z "$PLUGIN_DIR" ]; then
    error "Could not determine GStreamer plugin directory!"
    exit 1
fi

info "Target plugin directory: $PLUGIN_DIR"

# Check if we need sudo for system installation
SUDO=""
if [ "$INSTALL_MODE" = "system" ] && [ "$EUID" -ne 0 ]; then
    SUDO="sudo"
    warning "System installation requires sudo privileges"
fi

# Create plugin directory if it doesn't exist
if [ ! -d "$PLUGIN_DIR" ]; then
    info "Creating plugin directory..."
    if [ "$DRY_RUN" = false ]; then
        $SUDO mkdir -p "$PLUGIN_DIR"
    else
        info "[DRY RUN] Would create: $PLUGIN_DIR"
    fi
fi

# Check for ONNX Runtime library
ONNX_LIB="$PROJECT_ROOT/target/$BUILD_TYPE/libonnxruntime.so"
if [ ! -f "$ONNX_LIB" ] && [ ! -f "${ONNX_LIB}.1.16.3" ]; then
    warning "ONNX Runtime library not found in build directory"
    warning "Make sure ONNX Runtime is installed system-wide or set LD_LIBRARY_PATH"
fi

# Copy plugin
info "Installing plugin..."
if [ "$DRY_RUN" = false ]; then
    $SUDO cp "$PLUGIN_FILE" "$PLUGIN_DIR/"
    info "Plugin installed successfully"
    
    # Copy ONNX Runtime if present and doing user installation
    if [ "$INSTALL_MODE" = "user" ] && [ -f "$ONNX_LIB" ]; then
        cp "$ONNX_LIB"* "$PLUGIN_DIR/" 2>/dev/null || true
        info "ONNX Runtime library copied"
    fi
else
    info "[DRY RUN] Would copy $PLUGIN_FILE to $PLUGIN_DIR/"
fi

# Update library cache for system installation
if [ "$INSTALL_MODE" = "system" ] && [ "$DRY_RUN" = false ]; then
    info "Updating library cache..."
    $SUDO ldconfig
fi

# Clear plugin cache
CACHE_DIR="$HOME/.cache/gstreamer-1.0"
if [ -d "$CACHE_DIR" ]; then
    info "Clearing plugin cache..."
    if [ "$DRY_RUN" = false ]; then
        rm -rf "$CACHE_DIR"/*
    else
        info "[DRY RUN] Would clear: $CACHE_DIR"
    fi
fi

# Set GST_PLUGIN_PATH for user installation
if [ "$INSTALL_MODE" = "user" ]; then
    export GST_PLUGIN_PATH="$PLUGIN_DIR:$GST_PLUGIN_PATH"
    info "GST_PLUGIN_PATH set to include: $PLUGIN_DIR"
fi

# Verify installation
info "Verifying installation..."
if [ "$DRY_RUN" = false ]; then
    if gst-inspect-1.0 cpuinfer &> /dev/null; then
        info "Plugin verification successful!"
        
        if [ "$VERBOSE" = true ]; then
            echo ""
            info "Plugin details:"
            gst-inspect-1.0 cpuinfer
        fi
        
        echo ""
        info "Installation complete!"
        info "You can now use the plugin with:"
        info "  gst-launch-1.0 ... ! cpudetector ! ..."
        
        if [ "$INSTALL_MODE" = "user" ]; then
            echo ""
            info "To make the plugin permanently available, add this to your ~/.bashrc:"
            info "  export GST_PLUGIN_PATH=\"$PLUGIN_DIR:\$GST_PLUGIN_PATH\""
        fi
    else
        error "Plugin verification failed!"
        info "Try running with GST_DEBUG=3 gst-inspect-1.0 cpuinfer for more information"
        exit 1
    fi
else
    info "[DRY RUN] Would verify plugin with: gst-inspect-1.0 cpuinfer"
fi
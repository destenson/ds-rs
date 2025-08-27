# PRP-52: Implement nvinfer-Compatible Properties and Configuration for cpuinfer

## STATUS: PARTIALLY COMPLETE (2025-08-27)

### Progress Made:
✅ Added all nvinfer-compatible properties to Settings struct
✅ Implemented property getters/setters in GObject interface
✅ Created config.rs with INI parser for nvinfer config files
✅ Config file loading integrated with property system
✅ Properties include: batch-size, unique-id, process-mode, output-tensor-meta

### Remaining Work:
❌ Properties not showing in gst-inspect-1.0 output
❌ Batch processing logic not implemented
❌ Secondary mode (processing crops) not implemented
❌ Metadata attachment to buffers not implemented

## Problem Statement

The cpuinfer element needs to be compatible with NVIDIA's nvinfer element to serve as a drop-in CPU-based alternative. This requires implementing the same property interface, configuration file parsing, and metadata output format that nvinfer uses. Applications using nvinfer should be able to switch to cpuinfer with minimal changes.

## Research & References

### nvinfer Documentation
- https://docs.nvidia.com/metropolis/deepstream/dev-guide/text/DS_sample_custom_gstream.html - DeepStream custom GStreamer plugins
- Configuration files in `../NVIDIA-AI-IOT--deepstream_reference_apps/runtime_source_add_delete/*.txt`

### nvinfer Properties Analysis
From `../NVIDIA-AI-IOT--deepstream_reference_apps/runtime_source_add_delete/deepstream_test_rt_src_add_del.c`:
- `config-file-path`: Path to configuration file (string)
- `batch-size`: Number of frames to batch (uint)
- `unique-id`: Unique identifier for element (uint)
- `process-mode`: Primary (1) or secondary (2) mode
- `output-tensor-meta`: Output tensor metadata (boolean)

### Configuration File Format
From `../NVIDIA-AI-IOT--deepstream_reference_apps/runtime_source_add_delete/dstest_pgie_config.txt`:
- INI-style format with sections
- `[property]` section for main configuration
- `[class-attrs-all]` section for detection parameters
- Support for ONNX models via `onnx-file` property

## Current State Analysis

### What Exists
- Basic GStreamer element structure in `crates/cpuinfer/src/cpudetector/`
- ONNX model loading in `crates/cpuinfer/src/detector.rs`
- Detection output generation

### What's Missing
- Property interface matching nvinfer
- Configuration file parser
- Batch processing support
- Metadata output in DeepStream format
- Primary/secondary mode distinction

## Implementation Blueprint

### Phase 1: Property Interface

#### 1.1 Define Properties
**File**: `crates/cpuinfer/src/cpudetector/imp.rs`
**Properties to Add**:
```
- config-file-path (String, readable/writable)
- batch-size (u32, readable/writable, default: 1)
- unique-id (u32, readable/writable, default: 0)  
- process-mode (u32, readable/writable, default: 1)
- output-tensor-meta (bool, readable/writable, default: false)
- model-engine-file (String, readable/writable)
- gpu-id (u32, readable/writable, default: 0) - ignored for CPU
```

**Pattern to Follow**: Use `glib::ParamSpec` definitions similar to `../gst-plugins-rs/tutorial/src/rgb2gray/imp.rs`

#### 1.2 Property Implementation
**Location**: `crates/cpuinfer/src/cpudetector/imp.rs`
**Methods to Implement**:
- `properties()` - Return property specifications
- `property()` - Get property values
- `set_property()` - Set property values and trigger reinitialization if needed

### Phase 2: Configuration File Parser

#### 2.1 Config Structure Definition
**File**: `crates/cpuinfer/src/config.rs` (new file)
**Structures**:
```
InferConfig {
  // [property] section
  onnx_file: Option<String>
  model_engine_file: Option<String>
  labelfile_path: Option<String>
  batch_size: u32
  process_mode: u32
  num_detected_classes: u32
  interval: u32
  unique_id: u32
  network_mode: u32
  cluster_mode: u32
  
  // [class-attrs-all] section  
  pre_cluster_threshold: f32
  nms_iou_threshold: f32
  topk: u32
}
```

#### 2.2 INI Parser Implementation
**File**: `crates/cpuinfer/src/config.rs`
**Dependencies**: Add `configparser = "3.0"` or similar INI parsing crate
**Functions**:
- `parse_config_file(path: &str) -> Result<InferConfig>`
- `validate_config(config: &InferConfig) -> Result<()>`

### Phase 3: Batch Processing

#### 3.1 Batch Accumulation
**File**: `crates/cpuinfer/src/cpudetector/imp.rs`
**Logic**:
- Accumulate buffers until batch-size is reached
- Process batch through detector
- Distribute results back to individual buffers

#### 3.2 Batch Metadata
**Pattern**: Similar to how nvstreammux creates batch metadata
- Create batch metadata structure
- Attach frame metadata for each buffer in batch
- Propagate metadata downstream

### Phase 4: Metadata Output Format

#### 4.1 Define Metadata Structures
**File**: `crates/cpuinfer/src/metadata.rs` (new file)
**Structures to Match DeepStream**:
```
NvDsObjectMeta - Detection results
NvDsClassifierMeta - Classification results  
NvDsBatchMeta - Batch-level metadata
NvDsFrameMeta - Frame-level metadata
```

#### 4.2 Metadata Attachment
**Location**: `crates/cpuinfer/src/cpudetector/imp.rs`
**Process**:
- After detection, create metadata structures
- Attach to GstBuffer using GstMeta API
- Use same meta API names as DeepStream for compatibility

### Phase 5: Primary vs Secondary Mode

#### 5.1 Mode Handling
**File**: `crates/cpuinfer/src/cpudetector/imp.rs`
**Behavior Differences**:
- Primary mode (1): Process full frame, detect objects
- Secondary mode (2): Process crops from upstream detections
- Check for upstream object metadata in secondary mode

#### 5.2 Secondary Mode Processing
**Logic**:
- Read object metadata from upstream
- Crop regions based on bounding boxes
- Run inference on crops
- Attach classifier metadata to existing objects

## Configuration Compatibility Matrix

| nvinfer Property | cpuinfer Support | Notes |
|-----------------|------------------|-------|
| onnx-file | Full | Primary model format |
| model-engine-file | Ignored | No TensorRT on CPU |
| batch-size | Full | Batch processing |
| num-detected-classes | Full | Class count |
| labelfile-path | Full | Class labels |
| process-mode | Full | Primary/Secondary |
| unique-id | Full | Element identifier |
| gpu-id | Ignored | Always CPU |
| network-mode | Partial | FP32 only on CPU |
| cluster-mode | Full | NMS modes |

## Testing with Existing Pipelines

### Test Pipeline from DeepStream
The element should work with existing DeepStream pipelines by replacing nvinfer:
```
filesrc ! qtdemux ! h264parse ! nvv4l2decoder ! nvstreammux ! cpuinfer config-file-path=config.txt ! ...
```

### Compatibility Validation
Test with configuration files from:
- `../NVIDIA-AI-IOT--deepstream_reference_apps/runtime_source_add_delete/dstest_pgie_config.txt`
- Modify paths to point to ONNX models instead of TensorRT engines

## Validation Gates

```bash
# Build with new properties
cargo build --release -p cpuinfer

# Check properties are exposed
gst-inspect-1.0 cpuinfer | grep -E "config-file-path|batch-size|unique-id"

# Test config file parsing
cat > test_config.txt << EOF
[property]
onnx-file=model.onnx
batch-size=4
num-detected-classes=80
process-mode=1

[class-attrs-all]
pre-cluster-threshold=0.4
nms-iou-threshold=0.5
EOF

# Test pipeline with config
gst-launch-1.0 videotestsrc num-buffers=10 ! \
  cpuinfer config-file-path=test_config.txt ! \
  fakesink

# Verify batch processing
gst-launch-1.0 videotestsrc num-buffers=10 ! \
  cpuinfer batch-size=4 ! \
  fakesink

# Check metadata output
GST_DEBUG=cpuinfer:5 gst-launch-1.0 videotestsrc ! \
  cpuinfer output-tensor-meta=true ! \
  fakesink 2>&1 | grep -i meta
```

## Success Criteria

- All listed properties are accessible via gst-inspect-1.0
- Configuration files parse correctly
- Batch processing accumulates and processes frames
- Metadata format matches DeepStream expectations
- Primary and secondary modes work correctly
- Existing DeepStream applications work with cpuinfer

## Risk Assessment

- **High Risk**: Metadata format incompatibility
- **Mitigation**: Carefully study DeepStream metadata structures
- **Medium Risk**: Configuration parsing differences
- **Mitigation**: Support subset of nvinfer options initially

## Dependencies to Add

In `crates/cpuinfer/Cargo.toml`:
```toml
configparser = "3.0"  # or similar INI parser
```

## Estimated Effort

- Property implementation: 3-4 hours
- Config parser: 2-3 hours
- Batch processing: 4-5 hours
- Metadata format: 3-4 hours
- Testing and debugging: 3-4 hours
- Total: 15-20 hours

## References from Codebase

- Property patterns: `../gst-plugins-rs/tutorial/src/rgb2gray/imp.rs`
- Metadata examples: `../prominenceai--deepstream-services-library/src/` 
- Config examples: `../NVIDIA-AI-IOT--deepstream_reference_apps/runtime_source_add_delete/*.txt`

## Next Steps

After implementing these features, create PRP-53 for:
- Performance optimization for CPU inference
- Support for multiple model formats beyond ONNX
- Advanced features like model switching at runtime

## Confidence Score: 6/10

The implementation is complex due to DeepStream metadata compatibility requirements. The score would be higher with access to DeepStream header files defining exact metadata structures. The main challenge is ensuring metadata format compatibility without breaking existing DeepStream applications.
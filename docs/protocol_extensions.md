# OpenIGTLink Protocol Extensions

## Overview

This document outlines backward-compatible extensions to the OpenIGTLink protocol that address current limitations while maintaining 100% compatibility with existing Version 2 and Version 3 implementations.

All proposed extensions leverage OpenIGTLink Version 3's Extended Header and Metadata sections, ensuring that implementations without these features continue to function normally.

## Background: Current Protocol Limitations

Based on research and analysis of the OpenIGTLink protocol and modern medical imaging requirements, several areas for improvement have been identified:

### 1. Performance Issues
- **Large image transfers block other data**: When transferring high-resolution images, smaller messages (transforms, status) are blocked until the image completes
- **CPU-intensive string operations**: Device name and data type comparisons consume significant CPU time
- **No compression negotiation**: Compression support is message-type specific and cannot be negotiated

### 2. Streaming Quality
- **No adaptive quality control**: Real-time surgical video cannot adjust quality based on network conditions
- **Fixed frame rates**: Cannot dynamically adjust streaming parameters during transmission
- **No bandwidth management**: No mechanism to prioritize critical data during network congestion

### 3. Error Recovery
- **All-or-nothing transfers**: Partial transfer failures require complete retransmission
- **No state recovery**: Connection loss requires full session restart
- **Limited checkpointing**: No intermediate save points for long transfers

### 4. Network Efficiency
- **High overhead for small messages**: Each message has 58-byte header overhead
- **No pipelining**: Sequential request-response pattern adds latency
- **No batching**: Multiple small messages cannot be combined

## Proposed Extensions

All extensions use OpenIGTLink Version 3's Extended Header (customizable per implementation) and Metadata (key-value pairs) sections to maintain backward compatibility.

---

## Extension 1: Message Priority and QoS (Quality of Service)

### Motivation
Enable prioritization of critical messages (transforms, commands) over bulk data (images, video) during network congestion.

### Design

```rust
/// Priority levels for message transmission
pub struct MessagePriority {
    /// Priority level: 0 (lowest) to 255 (highest)
    /// Recommended levels:
    ///   255: Emergency stop commands
    ///   200: Real-time transforms
    ///   150: Status updates
    ///   100: Standard commands
    ///   50: Images
    ///   10: Video frames
    ///   0: Background data
    pub level: u8,

    /// Transmission timeout in milliseconds
    /// If message cannot be sent within timeout, it may be dropped
    pub timeout_ms: u32,
}

/// Quality of Service flags for message handling
pub struct QoSFlags {
    /// Allow dropping this message if network is congested
    pub allow_drop: bool,

    /// Enable automatic compression for this message
    pub compress: bool,

    /// For large messages, split into chunks of this size
    /// None = send as single message
    pub chunk_size: Option<u32>,

    /// Request delivery confirmation from receiver
    pub require_ack: bool,
}
```

### Implementation in Extended Header

```
Byte offset | Size | Description
------------|------|-------------
0-1         | 2    | Extended header size (including QoS fields)
2-3         | 2    | QoS flags bitmask
4           | 1    | Priority level (0-255)
5-8         | 4    | Timeout in milliseconds
```

QoS Flags Bitmask:
- Bit 0: allow_drop
- Bit 1: compress
- Bit 2: require_ack
- Bits 3-15: Reserved (must be 0)

### Backward Compatibility
- Version 2 implementations: Ignore extended header, process message normally
- Version 3 implementations without QoS: Ignore QoS fields, process message normally
- Version 3 implementations with QoS: Use priority and flags for optimization

### Usage Example

```rust
use openigtlink_rs::io::IgtlClient;
use openigtlink_rs::messages::TransformMessage;
use openigtlink_rs::protocol::{MessagePriority, QoSFlags};

let mut client = IgtlClient::connect("localhost:18944")?;

// High-priority transform for surgical navigation
let mut transform = TransformMessage::new("SurgicalTool");
transform.set_matrix(&tool_matrix);
transform.set_priority(MessagePriority {
    level: 200,
    timeout_ms: 10,  // Drop if can't send within 10ms
});
transform.set_qos(QoSFlags {
    allow_drop: false,  // Never drop critical navigation data
    compress: false,    // Low-latency more important than size
    chunk_size: None,
    require_ack: true,
});

client.send(&transform)?;

// Low-priority background image
let mut image = ImageMessage::new("BackgroundScan");
image.set_image_data(&scan_data);
image.set_priority(MessagePriority {
    level: 50,
    timeout_ms: 5000,
});
image.set_qos(QoSFlags {
    allow_drop: true,   // OK to drop if network congested
    compress: true,     // Compress to save bandwidth
    chunk_size: Some(65536),  // Send in 64KB chunks
    require_ack: false,
});

client.send(&image)?;
```

---

## Extension 2: Adaptive Streaming Control

### Motivation
Enable dynamic adjustment of streaming quality based on real-time network conditions, critical for surgical video streaming where quality must adapt to available bandwidth.

### Design

```rust
/// Network condition hints for adaptive streaming
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkCondition {
    /// >100 Mbps bandwidth, <10ms latency
    Excellent,
    /// >50 Mbps bandwidth, <50ms latency
    Good,
    /// >10 Mbps bandwidth, <100ms latency
    Fair,
    /// <10 Mbps bandwidth or >100ms latency
    Poor,
}

/// Streaming control parameters
pub struct StreamingControl {
    /// Target frames per second (None = sender decides)
    pub target_fps: Option<f32>,

    /// Maximum bitrate in Mbps (None = no limit)
    pub max_bitrate_mbps: Option<f32>,

    /// Enable adaptive quality based on network conditions
    pub adaptive_quality: bool,

    /// Current network condition hint
    pub network_hint: NetworkCondition,

    /// Minimum acceptable quality (0-100)
    pub min_quality: u8,
}

/// Extended STT_STREAM message with quality control
pub struct StartStreamEx {
    pub device_name: String,
    pub control: StreamingControl,
}
```

### Implementation in Metadata

When sending `STT_STREAM` or `STT_VIDEO` commands, add metadata:

```
Key                 | Type   | Description
--------------------|--------|----------------------------------
target_fps          | float  | Desired frames per second
max_bitrate         | float  | Maximum Mbps
adaptive_quality    | int    | 1=enabled, 0=disabled
network_condition   | int    | 0=Poor, 1=Fair, 2=Good, 3=Excellent
min_quality         | int    | 0-100, minimum acceptable quality
```

### Sender Behavior

The streaming sender monitors these parameters and adjusts:
- **Excellent conditions**: Full resolution, max quality, target FPS
- **Good conditions**: Full resolution, high quality, slight FPS reduction
- **Fair conditions**: Reduced resolution or quality, lower FPS
- **Poor conditions**: Minimum quality, reduced FPS, possible frame dropping

### Usage Example

```rust
use openigtlink_rs::messages::CommandMessage;
use openigtlink_rs::protocol::{StreamingControl, NetworkCondition};

// Request adaptive streaming with quality constraints
let control = StreamingControl {
    target_fps: Some(30.0),
    max_bitrate_mbps: Some(50.0),
    adaptive_quality: true,
    network_hint: NetworkCondition::Good,
    min_quality: 60,  // Never go below 60% quality
};

let cmd = CommandMessage::start_stream_with_control("VideoCamera", control);
client.send(&cmd)?;

// Later, receiver can update network hint
let update = CommandMessage::update_stream_quality(
    "VideoCamera",
    NetworkCondition::Fair,  // Network degraded
);
client.send(&update)?;
```

---

## Extension 3: Multi-Resolution Image Support

### Motivation
Enable progressive image loading where low-resolution preview arrives quickly, followed by higher resolutions as bandwidth allows. Critical for large medical images (CT, MRI scans).

### Design

```rust
/// Multi-resolution image level
pub struct ImageLevel {
    /// Resolution [width, height, depth]
    pub resolution: [u16; 3],

    /// Quality factor 0-100 (JPEG-style)
    pub quality_factor: u8,

    /// Compressed or raw image data
    pub data: Vec<u8>,
}

/// Multi-resolution image message
pub struct MultiResolutionImage {
    pub device_name: String,

    /// Multiple resolution levels, ordered from lowest to highest
    pub levels: Vec<ImageLevel>,

    /// Index of base level (minimum required for viewing)
    pub base_level: usize,

    /// Scalar type (same as IMAGE message)
    pub scalar_type: ScalarType,
}
```

### New Message Type: `MRES_IMG`

Extends the standard `IMAGE` message with multiple resolution levels.

#### Message Structure

```
HEADER (58 bytes, standard OpenIGTLink v3)

EXTENDED HEADER:
  Byte 0-1:   Extended header size
  Byte 2:     Number of resolution levels
  Byte 3:     Base level index
  Byte 4:     Scalar type
  Byte 5-7:   Reserved

CONTENT:
  For each level:
    Bytes 0-1:   Width
    Bytes 2-3:   Height
    Bytes 4-5:   Depth
    Byte 6:      Quality factor
    Bytes 7-10:  Data size
    Bytes 11+:   Image data

METADATA (optional):
  compression: "none", "jpeg", "deflate"
  color_space: "RGB", "RGBA", "Grayscale"
```

### Sender Behavior

1. Generate multiple resolution levels (e.g., 25%, 50%, 100%)
2. Mark lowest resolution as base_level
3. Send all levels in single message
4. Receiver can display base level immediately

### Usage Example

```rust
use openigtlink_rs::messages::MultiResolutionImage;
use openigtlink_rs::protocol::ImageLevel;

// Create multi-resolution CT scan
let mut mri = MultiResolutionImage::new("CTScan");

// Level 0: 128x128x64 preview (quick to transfer)
mri.add_level(ImageLevel {
    resolution: [128, 128, 64],
    quality_factor: 70,
    data: generate_preview(&full_image, 0.25),
});

// Level 1: 256x256x128 medium quality
mri.add_level(ImageLevel {
    resolution: [256, 256, 128],
    quality_factor: 85,
    data: generate_preview(&full_image, 0.5),
});

// Level 2: 512x512x256 full resolution
mri.add_level(ImageLevel {
    resolution: [512, 512, 256],
    quality_factor: 100,
    data: full_image.clone(),
});

mri.set_base_level(0);  // Preview sufficient for initial view
client.send(&mri)?;
```

### Backward Compatibility

Receivers that don't understand `MRES_IMG`:
- Ignore the message type (unknown message types are skipped)
- Sender can fallback to standard `IMAGE` message if `GET_CAPABIL` doesn't list `MRES_IMG`

---

## Extension 4: Transfer Checkpointing

### Motivation
Enable resumption of large data transfers after network interruptions without retransmitting already-received data.

### Design

```rust
/// Checkpoint for resumable transfers
pub struct TransferCheckpoint {
    /// Unique transfer identifier
    pub transfer_id: u64,

    /// Number of bytes successfully received
    pub byte_offset: u64,

    /// Timestamp of checkpoint (microseconds since epoch)
    pub timestamp: u64,

    /// CRC64 checksum up to byte_offset
    pub checksum: u64,
}

/// Request checkpoint creation
pub struct CheckpointRequest {
    pub transfer_id: u64,
}

/// Resume transfer from checkpoint
pub struct ResumeTransfer {
    pub transfer_id: u64,
    pub checkpoint: TransferCheckpoint,
}
```

### New Command Messages

#### `RTS_CHKPT` - Request Checkpoint

Receiver requests sender to create checkpoint during long transfer.

Metadata fields:
- `transfer_id`: Unique ID for this transfer
- `byte_offset`: Current bytes received
- `timestamp`: When checkpoint requested

#### `RTS_RESUME` - Resume Transfer

After reconnection, receiver requests sender to resume from checkpoint.

Metadata fields:
- `transfer_id`: Transfer to resume
- `byte_offset`: Resume from this offset
- `checksum`: Expected CRC64 up to offset

### Protocol Flow

```
Normal transfer:
  Sender: IMAGE message (transfer_id=123, total_size=10MB)
  Receiver: Receiving... (5MB received)
  [Network interruption]

Resume transfer:
  Receiver: RTS_RESUME (transfer_id=123, byte_offset=5MB, checksum=0x...)
  Sender: Validates checksum
  Sender: IMAGE message (transfer_id=123, offset=5MB, remaining 5MB data)
  Receiver: Complete
```

### Usage Example

```rust
use openigtlink_rs::protocol::{TransferCheckpoint, ResumeTransfer};

// Sender: Add transfer_id to large message
let mut image = ImageMessage::new("LargeScan");
image.set_transfer_id(12345);
image.set_image_data(&large_scan);  // 100MB

match client.send(&image) {
    Ok(_) => println!("Transfer complete"),
    Err(e) if e.is_network_error() => {
        // Save checkpoint for resume
        let checkpoint = image.get_checkpoint();
        save_checkpoint(&checkpoint)?;
    }
    Err(e) => return Err(e),
}

// After reconnection
let checkpoint = load_checkpoint()?;
let resume = CommandMessage::resume_transfer(checkpoint);
client.send(&resume)?;

// Sender will resume from checkpoint offset
```

---

## Extension 5: Message Batching

### Motivation
Reduce overhead when sending many small messages (transforms, status updates) by combining into single network transmission.

### Design

```rust
/// Message batching options
pub struct BatchMessage {
    /// Messages to send in batch
    pub messages: Vec<Box<dyn IgtlMessage>>,

    /// Processing order requirement
    pub ordering: BatchOrdering,
}

/// Batch processing order
#[derive(Debug, Clone, Copy)]
pub enum BatchOrdering {
    /// Messages must be processed in order
    Sequential,

    /// Messages can be processed in parallel
    Parallel,

    /// Order doesn't matter
    Unordered,
}
```

### New Message Type: `BATCH`

Contains multiple OpenIGTLink messages in single transmission.

#### Message Structure

```
HEADER (58 bytes)
  Type: "BATCH"

EXTENDED HEADER:
  Byte 0-1:   Extended header size
  Byte 2:     Ordering (0=Sequential, 1=Parallel, 2=Unordered)
  Byte 3:     Number of messages in batch
  Bytes 4-7:  Reserved

CONTENT:
  For each message:
    Bytes 0-1:   Message size (including header)
    Bytes 2+:    Complete OpenIGTLink message (HEADER + BODY)
```

### Benefits

- **Reduced network overhead**: One TCP send instead of many
- **Lower latency**: Avoid TCP Nagle algorithm delays
- **Better throughput**: More efficient use of network bandwidth

### Usage Example

```rust
use openigtlink_rs::protocol::{BatchMessage, BatchOrdering};

// Create batch of transform updates
let mut batch = BatchMessage::new(BatchOrdering::Unordered);

// Add multiple tool positions
for (name, matrix) in tool_positions {
    let mut transform = TransformMessage::new(&name);
    transform.set_matrix(&matrix);
    batch.add(transform);
}

// Send all transforms in single transmission
client.send_batch(&batch)?;

// Receiver processes all transforms
// For Unordered batches, can process in parallel
```

### Backward Compatibility

- Receivers that don't understand `BATCH`: Skip the message
- Use `GET_CAPABIL` to check if receiver supports batching
- Fallback to individual messages if unsupported

---

## Implementation Strategy

### Phase 1: Foundation (Priority)
1. Add Extended Header v3 support infrastructure
2. Implement message priority and QoS
3. Add capability negotiation for extensions

### Phase 2: Streaming (Next)
1. Adaptive streaming control
2. Network condition monitoring
3. Dynamic quality adjustment

### Phase 3: Efficiency (Future)
1. Multi-resolution images
2. Transfer checkpointing
3. Message batching

### Feature Detection

Extend `GET_CAPABIL` response to include:

```
CAPABIL message metadata:
  qos_support: "true"/"false"
  adaptive_streaming: "true"/"false"
  multi_resolution: "true"/"false"
  checkpointing: "true"/"false"
  batching: "true"/"false"
```

Clients query capabilities and enable features only if supported.

### Version Negotiation

```rust
// Query receiver capabilities
let capabil = client.get_capabilities()?;

if capabil.supports("qos_support") {
    // Enable QoS features
    message.set_priority(MessagePriority { level: 200, timeout_ms: 10 });
}

if capabil.supports("adaptive_streaming") {
    // Use adaptive streaming
    let control = StreamingControl { /* ... */ };
    client.start_stream_with_control("Camera", control)?;
}
```

---

## Backward Compatibility Guarantees

All extensions maintain strict backward compatibility:

1. **Version 2 implementations**:
   - Ignore Extended Header completely
   - Process message body normally
   - No breaking changes

2. **Version 3 implementations without extensions**:
   - Skip unknown Extended Header fields
   - Ignore unknown Metadata keys
   - Process known fields normally

3. **Version 3 implementations with extensions**:
   - Use Extended Header for optimization
   - Leverage Metadata for enhanced features
   - Fallback gracefully if peer doesn't support

4. **Binary protocol unchanged**:
   - Header format identical
   - CRC64 calculation same
   - Network byte order preserved

## Testing Strategy

1. **Interoperability tests**:
   - Test against C++ OpenIGTLink reference implementation
   - Verify extensions are ignored gracefully
   - Confirm no protocol violations

2. **Feature tests**:
   - Test each extension independently
   - Verify fallback behavior
   - Measure performance improvements

3. **Stress tests**:
   - High message volume with priorities
   - Network congestion scenarios
   - Large transfers with checkpointing

## Performance Expectations

Based on protocol analysis and modern network capabilities:

- **Message Priority**: 10-50% latency reduction for high-priority messages under load
- **Adaptive Streaming**: 30-70% better quality in variable network conditions
- **Multi-Resolution**: 5-10x faster initial image display
- **Checkpointing**: 90%+ bandwidth savings on transfer resume
- **Batching**: 20-40% throughput increase for small messages

## References

1. OpenIGTLink Protocol v3 Specification: http://openigtlink.org/protocols/v3
2. "OpenIGTLink: an open network protocol for image-guided therapy environment" (2009)
3. Modern medical imaging streaming requirements research (2024)
4. Real-time surgical video streaming requirements (2020-2024)

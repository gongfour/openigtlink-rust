# Query and Streaming Control Messages

This guide explains how to use OpenIGTLink query and streaming control messages for C++ OpenIGTLink server compatibility.

## Overview

Query and streaming control messages enable:
- **Query protocol**: Request single data items from a server (GET_*, responds with data message)
- **Streaming control**: Start/stop continuous data streams (STT_*, STP_*, RTS_*)
- **C++ compatibility**: Full compatibility with 3D Slicer, PLUS Toolkit, and other C++ OpenIGTLink servers

## Message Types

### GET_* (Query Messages)

Request a single data item from the server. The server responds with the corresponding data message.

| Message Type | Response Type | Purpose |
|-------------|---------------|---------|
| `GET_CAPABIL` | CAPABILITY | Query supported message types |
| `GET_STATUS` | STATUS | Query device/system status |
| `GET_TRANSFOR` | TRANSFORM | Query current transformation |
| `GET_IMAGE` | IMAGE | Query current image |
| `GET_TDATA` | TDATA | Query tracking data |
| `GET_POINT` | POINT | Query point data |
| `GET_IMGMETA` | IMGMETA | Query image metadata |
| `GET_LBMETA` | LBMETA | Query label metadata |

### STT_* (Start Streaming)

Request the server to start streaming data at a specified rate.

| Message Type | Stream Type | Parameters |
|-------------|-------------|------------|
| `STT_TDATA` | TDATA | `resolution` (ms), `coordinate_name` (32 bytes) |
| Other STT_* | Various | No parameters (empty body) |

### STP_* (Stop Streaming)

Request the server to stop streaming data.

| Message Type | Purpose |
|-------------|---------|
| `STP_TDATA` | Stop TDATA streaming |
| `STP_IMAGE` | Stop IMAGE streaming |
| `STP_TRANSFOR` | Stop TRANSFORM streaming |
| `STP_POSITION` | Stop POSITION streaming |
| `STP_QTDATA` | Stop QTDATA streaming |
| `STP_NDARRAY` | Stop NDARRAY streaming |

### RTS_* (Ready-to-Send Response)

Server acknowledgment that it's ready to send data.

| Message Type | Response Data |
|-------------|---------------|
| `RTS_TDATA` | `status` (u16): 0=error, 1=ok |
| Other RTS_* | STATUS message (code: 0/1) |

## Usage Examples

### Example 1: Query Server Capabilities

```rust
use openigtlink_rust::io::ClientBuilder;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{GetCapabilityMessage, CapabilityMessage};

// Connect to C++ OpenIGTLink server
let mut client = ClientBuilder::new()
    .tcp("192.168.1.100:18944")
    .sync()
    .build()?;

// Send GET_CAPABIL query
let query = GetCapabilityMessage;
let msg = IgtlMessage::new(query, "RustClient")?;
client.send(&msg)?;

// Receive CAPABILITY response
let response: IgtlMessage<CapabilityMessage> = client.receive()?;
println!("Supported types: {:?}", response.content.types);
```

### Example 2: Start Tracking Data Stream

```rust
use openigtlink_rust::protocol::types::{
    StartTDataMessage, RtsTDataMessage, TDataMessage, StopTDataMessage
};

// 1. Request streaming start
let start_stream = StartTDataMessage {
    resolution: 50,                      // 50ms = 20 Hz update rate
    coordinate_name: "RAS".to_string(),  // Right-Anterior-Superior coordinate system
};

let msg = IgtlMessage::new(start_stream, "RustClient")?;
client.send(&msg)?;

// 2. Receive acknowledgment
let ack: IgtlMessage<RtsTDataMessage> = client.receive()?;
if ack.content.status != 1 {
    panic!("Server rejected streaming request (status={})", ack.content.status);
}

// 3. Receive tracking data stream
for _ in 0..100 {
    let tdata: IgtlMessage<TDataMessage> = client.receive()?;

    for element in &tdata.content.elements {
        let x = element.matrix[0][3];
        let y = element.matrix[1][3];
        let z = element.matrix[2][3];

        println!("{}: ({:.2}, {:.2}, {:.2})", element.name, x, y, z);
    }
}

// 4. Stop streaming
let stop_stream = StopTDataMessage;
let msg = IgtlMessage::new(stop_stream, "RustClient")?;
client.send(&msg)?;
```

### Example 3: Error Handling

```rust
use openigtlink_rust::error::IgtlError;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{CapabilityMessage, GetCapabilityMessage};
use std::io::ErrorKind;
use std::time::Duration;

client.set_read_timeout(Some(Duration::from_secs(5)))?;

let query_msg = IgtlMessage::new(GetCapabilityMessage, "RustClient")?;
match client.send(&query_msg) {
    Ok(_) => match client.receive::<CapabilityMessage>() {
        Ok(response) => {
            println!("Supported types: {:?}", response.content.types);
        }
        Err(IgtlError::Io(e)) if e.kind() == ErrorKind::WouldBlock => {
            eprintln!("Server did not respond in time");
        }
        Err(e) => {
            eprintln!("Receive error: {}", e);
        }
    },
    Err(e) => {
        eprintln!("Send error: {}", e);
    }
}
```

## Python Binding Usage

When wrapping this library for Python, the query/streaming protocol enables intuitive APIs:

### PyO3 Example

```python
import openigtlink_rust as igtl

# Connect to C++ OpenIGTLink server (3D Slicer)
client = igtl.Client("192.168.1.100:18944")

# Query capabilities
capabilities = client.get_capability()
print(f"Supported types: {capabilities}")

# Start tracking stream
stream = client.start_tracking_stream(
    resolution=50,           # 50ms update rate
    coordinate="RAS"         # Anatomical coordinate system
)

# Iterate over tracking data (Python iterator)
for tdata in stream:
    for tool in tdata.tools:
        print(f"{tool.name}: position={tool.position}, rotation={tool.rotation}")

    # Stop after 100 samples
    if stream.count >= 100:
        break

# Stream automatically stops when iterator exits
stream.stop()
```

### Python Async/Await Example

```python
import asyncio
import openigtlink_rust as igtl

async def track_tools():
    # Connect to server
    client = await igtl.AsyncClient.connect("192.168.1.100:18944")

    # Start streaming
    stream = await client.start_tracking_stream(resolution=50, coordinate="RAS")

    # Async iteration
    async for tdata in stream:
        for tool in tdata.tools:
            print(f"{tool.name}: {tool.position}")

        # Stop after 100 samples
        if stream.count >= 100:
            break

    await stream.stop()

asyncio.run(track_tools())
```

### Python Context Manager Pattern

```python
# Automatic cleanup with context manager
with client.start_tracking_stream(resolution=50, coordinate="RAS") as stream:
    for tdata in stream.take(100):  # Take first 100 samples
        process_tracking_data(tdata)
# Stream automatically stopped on exit
```

## C++ Server Compatibility

### 3D Slicer Integration

To use with 3D Slicer's OpenIGTLink module:

1. Load OpenIGTLink module in 3D Slicer
2. Create a server connector on port 18944
3. Run the Rust client:

```bash
cargo run --example query_streaming -- localhost:18944
```

### PLUS Toolkit Integration

To use with PLUS Toolkit tracking server:

1. Start PlusServer with your tracking configuration:
   ```bash
   PlusServer --config-file=PlusDeviceSet_Server_Sim_NwirePhantom.xml
   ```

2. Connect from Rust:
   ```rust
   use openigtlink_rust::io::ClientBuilder;

   let mut client = ClientBuilder::new()
       .tcp("localhost:18944")
       .sync()
       .build()?;
   ```

## Protocol Flow Diagram

```
Rust Client                    C++ Server (3D Slicer/PLUS)
    |                                    |
    |--- GET_CAPABIL ------------------->|
    |<-- CAPABILITY ---------------------|
    |                                    |
    |--- STT_TDATA (50ms, "RAS") ------->|
    |<-- RTS_TDATA (status=1) -----------|
    |                                    |
    |<-- TDATA --------------------------|  (continuous stream)
    |<-- TDATA --------------------------|
    |<-- TDATA --------------------------|
    |    ...                             |
    |                                    |
    |--- STP_TDATA --------------------->|
    |                                    |
    v                                    v
```

## Best Practices

### 1. Always Check RTS_* Acknowledgment

```rust
use openigtlink_rust::error::IgtlError;
use openigtlink_rust::protocol::message::IgtlMessage;

let ack: IgtlMessage<RtsTDataMessage> = client.receive()?;
if ack.content.status == 0 {
    return Err(IgtlError::InvalidHeader("Server rejected request".into()));
}
```

### 2. Use Timeouts for Query Messages

```rust
use openigtlink_rust::error::IgtlError;
use openigtlink_rust::protocol::types::StatusMessage;
use std::io::ErrorKind;
use std::time::Duration;

client.set_read_timeout(Some(Duration::from_secs(5)))?;
match client.receive::<StatusMessage>() {
    Ok(response) => { /* ... */ }
    Err(IgtlError::Io(e)) if e.kind() == ErrorKind::WouldBlock => {
        eprintln!("Timeout waiting for response");
    }
    Err(e) => eprintln!("Receive error: {}", e),
}
```

### 3. Handle Stream Interruptions

```rust
use openigtlink_rust::io::ClientBuilder;

loop {
    match client.receive() {
        Ok(tdata) => process_tracking_data(tdata),
        Err(IgtlError::Io(e)) if e.kind() == std::io::ErrorKind::ConnectionReset => {
            eprintln!("Connection lost, attempting reconnect...");
            client = ClientBuilder::new().tcp(server_addr).sync().build()?;
        }
        Err(e) => return Err(e),
    }
}
```

### 4. Stop Streams on Error

```rust
// Use RAII pattern
use openigtlink_rust::io::SyncIgtlClient;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::StopTDataMessage;

struct StreamGuard<'a> {
    client: &'a mut SyncIgtlClient,
}

impl Drop for StreamGuard<'_> {
    fn drop(&mut self) {
        let stop = StopTDataMessage;
        let _ = self.client.send(&IgtlMessage::new(stop, "Client").unwrap());
    }
}
```

## Message Type Name Limitations

Due to OpenIGTLink protocol specification (12-character limit):
- `GET_TRANSFORM` → `GET_TRANSFOR` (truncated)
- `STP_TRANSFORM` → `STP_TRANSFOR` (truncated)

This is compatible with the C++ implementation.

## See Also

- [query_streaming.rs example](../examples/query_streaming.rs) - Complete working example
- [cpp_compat.rs tests](../tests/cpp_compat.rs) - Byte-level compatibility tests
- [OpenIGTLink Protocol Specification](https://github.com/openigtlink/OpenIGTLink/blob/master/Documents/Protocol/index.md)

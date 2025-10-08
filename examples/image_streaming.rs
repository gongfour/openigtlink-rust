//! Medical Image Streaming Example
//!
//! This example demonstrates streaming medical images (CT, MRI, Ultrasound)
//! using the OpenIGTLink IMAGE message type.
//!
//! # Usage
//!
//! ```bash
//! # Stream CT scan (512x512x100 slices, 16-bit)
//! cargo run --example image_streaming ct
//!
//! # Stream MRI scan (256x256x60 slices, 32-bit float)
//! cargo run --example image_streaming mri
//!
//! # Stream real-time ultrasound (640x480, 8-bit, 30fps)
//! cargo run --example image_streaming ultrasound
//! ```
//!
//! Make sure to run the server first:
//! ```bash
//! cargo run --example server
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::{ClientBuilder, SyncIgtlClient};
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{CoordinateSystem, ImageMessage, ImageScalarType};
use std::env;
use std::thread;
use std::time::Duration;

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let scenario = parse_scenario();

    // Connect to server
    let mut client = ClientBuilder::new().tcp("127.0.0.1:18944").sync().build()?;
    println!("[INFO] Connected to OpenIGTLink server\n");

    // Execute imaging scenario
    match scenario.as_str() {
        "ct" => stream_ct_scan(&mut client)?,
        "mri" => stream_mri_scan(&mut client)?,
        "ultrasound" => stream_ultrasound(&mut client)?,
        _ => unreachable!(),
    }

    println!("\n[INFO] Streaming completed successfully");
    Ok(())
}

fn parse_scenario() -> String {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let scenario = args[1].to_lowercase();
        if ["ct", "mri", "ultrasound"].contains(&scenario.as_str()) {
            return scenario;
        }
    }

    // Default scenario
    println!("Usage: cargo run --example image_streaming [ct|mri|ultrasound]");
    println!("Defaulting to CT scan...\n");
    "ct".to_string()
}

/// Stream a CT scan (512x512 pixels, 100 slices, 16-bit)
///
/// Simulates a typical chest CT scan with:
/// - Resolution: 512x512 pixels per slice
/// - Slices: 100 (axial slices)
/// - Bit depth: 16-bit unsigned (Hounsfield units)
/// - Pixel spacing: 0.7mm x 0.7mm
/// - Slice thickness: 1.0mm
fn stream_ct_scan(client: &mut SyncIgtlClient) -> Result<()> {
    println!("=== CT Scan Streaming ===");
    println!("Resolution: 512x512x100");
    println!("Scalar Type: Uint16 (Hounsfield units)");
    println!("Spacing: 0.7mm x 0.7mm x 1.0mm\n");

    let total_slices = 100;

    for slice_num in 0..total_slices {
        // Generate simulated CT slice data
        let image_data = generate_ct_slice(slice_num);

        // Create IMAGE message
        let image = ImageMessage::new(ImageScalarType::Uint16, [512, 512, 1], image_data)?
            .with_coordinate(CoordinateSystem::LPS);

        // Send to server
        let msg = IgtlMessage::new(image, "CTScanner")?;
        client.send(&msg)?;

        print!("\r[CT] Sending slice {}/{}", slice_num + 1, total_slices);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Simulate scanner acquisition time (50ms per slice)
        thread::sleep(Duration::from_millis(50));
    }

    println!(); // New line after progress
    Ok(())
}

/// Stream an MRI scan (256x256 pixels, 60 slices, 32-bit float)
///
/// Simulates a brain MRI scan with:
/// - Resolution: 256x256 pixels per slice
/// - Slices: 60
/// - Bit depth: 32-bit float (normalized intensity)
/// - Pixel spacing: 1.0mm x 1.0mm
/// - Slice thickness: 2.0mm
fn stream_mri_scan(client: &mut SyncIgtlClient) -> Result<()> {
    println!("=== MRI Scan Streaming ===");
    println!("Resolution: 256x256x60");
    println!("Scalar Type: Float32 (normalized intensity)");
    println!("Spacing: 1.0mm x 1.0mm x 2.0mm\n");

    let total_slices = 60;

    for slice_num in 0..total_slices {
        // Generate simulated MRI slice data
        let image_data = generate_mri_slice(slice_num);

        // Create IMAGE message
        let image = ImageMessage::new(ImageScalarType::Float32, [256, 256, 1], image_data)?
            .with_coordinate(CoordinateSystem::RAS);

        // Send to server
        let msg = IgtlMessage::new(image, "MRIScanner")?;
        client.send(&msg)?;

        print!("\r[MRI] Sending slice {}/{}", slice_num + 1, total_slices);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Simulate scanner acquisition time (100ms per slice)
        thread::sleep(Duration::from_millis(100));
    }

    println!(); // New line after progress
    Ok(())
}

/// Stream real-time ultrasound (640x480 pixels, 8-bit, 30fps)
///
/// Simulates real-time ultrasound imaging with:
/// - Resolution: 640x480 pixels
/// - Frame rate: 30 fps
/// - Bit depth: 8-bit grayscale
/// - Duration: 5 seconds (150 frames)
fn stream_ultrasound(client: &mut SyncIgtlClient) -> Result<()> {
    println!("=== Ultrasound Streaming ===");
    println!("Resolution: 640x480");
    println!("Scalar Type: Uint8");
    println!("Frame Rate: 30 fps");
    println!("Duration: 5 seconds\n");

    let fps: usize = 30;
    let duration_sec: usize = 5;
    let total_frames = fps * duration_sec;
    let frame_interval = Duration::from_millis(1000 / fps as u64);

    for frame_num in 0..total_frames {
        let start_time = std::time::Instant::now();

        // Generate simulated ultrasound frame
        let image_data = generate_ultrasound_frame(frame_num);

        // Create IMAGE message
        let image = ImageMessage::new(ImageScalarType::Uint8, [640, 480, 1], image_data)?
            .with_coordinate(CoordinateSystem::RAS);

        // Send to server
        let msg = IgtlMessage::new(image, "UltrasoundProbe")?;
        client.send(&msg)?;

        print!(
            "\r[US] Streaming frame {}/{} @ {}fps",
            frame_num + 1,
            total_frames,
            fps
        );
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Maintain frame rate
        let elapsed = start_time.elapsed();
        if elapsed < frame_interval {
            thread::sleep(frame_interval - elapsed);
        }
    }

    println!(); // New line after progress
    Ok(())
}

/// Generate simulated CT slice data (512x512, 16-bit)
///
/// Creates a synthetic CT image with:
/// - Gradient pattern (simulating tissue density)
/// - Values in Hounsfield units range (-1024 to 3071)
fn generate_ct_slice(slice_num: usize) -> Vec<u8> {
    let width = 512;
    let height = 512;
    let mut data = Vec::with_capacity(width * height * 2); // 2 bytes per pixel (Uint16)

    for y in 0..height {
        for x in 0..width {
            // Create radial gradient pattern
            let center_x = width as f32 / 2.0;
            let center_y = height as f32 / 2.0;
            let dx = (x as f32 - center_x) / center_x;
            let dy = (y as f32 - center_y) / center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Simulate tissue density (Hounsfield units)
            // -1024 (air) to 3071 (dense bone)
            let mut value = ((1.0 - distance) * 1500.0 + 500.0) as i16;
            value = value.max(-1024).min(3071);

            // Add slice-dependent variation
            value += (slice_num as i16 * 10) % 200;

            // Convert to unsigned 16-bit (shift by 1024)
            let unsigned_value = (value + 1024) as u16;

            // Little-endian encoding
            data.push((unsigned_value & 0xFF) as u8);
            data.push((unsigned_value >> 8) as u8);
        }
    }

    data
}

/// Generate simulated MRI slice data (256x256, 32-bit float)
///
/// Creates a synthetic MRI image with:
/// - Normalized intensity values (0.0 to 1.0)
/// - Gradient pattern simulating brain tissue
fn generate_mri_slice(slice_num: usize) -> Vec<u8> {
    let width = 256;
    let height = 256;
    let mut data = Vec::with_capacity(width * height * 4); // 4 bytes per pixel (Float32)

    for y in 0..height {
        for x in 0..width {
            // Create circular gradient pattern
            let center_x = width as f32 / 2.0;
            let center_y = height as f32 / 2.0;
            let dx = (x as f32 - center_x) / center_x;
            let dy = (y as f32 - center_y) / center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Normalized intensity (0.0 to 1.0)
            let mut intensity = (1.0 - distance).max(0.0);

            // Add slice-dependent variation
            intensity *= 0.7 + 0.3 * ((slice_num as f32 * 0.1).sin() + 1.0) / 2.0;

            // Convert float32 to bytes (little-endian)
            let bytes = intensity.to_le_bytes();
            data.extend_from_slice(&bytes);
        }
    }

    data
}

/// Generate simulated ultrasound frame (640x480, 8-bit)
///
/// Creates a synthetic ultrasound image with:
/// - Speckle noise pattern
/// - Animated motion
fn generate_ultrasound_frame(frame_num: usize) -> Vec<u8> {
    let width = 640;
    let height = 480;
    let mut data = Vec::with_capacity(width * height);

    for y in 0..height {
        for x in 0..width {
            // Create speckle pattern with animation
            let phase = frame_num as f32 * 0.1;
            let nx = x as f32 * 0.05 + phase;
            let ny = y as f32 * 0.05;

            // Simple noise-like pattern
            let value = ((nx.sin() * ny.cos() + 1.0) * 127.5) as u8;

            // Add fan-shaped attenuation (typical in ultrasound)
            let attenuation = 1.0 - (y as f32 / height as f32) * 0.3;
            let final_value = (value as f32 * attenuation) as u8;

            data.push(final_value);
        }
    }

    data
}

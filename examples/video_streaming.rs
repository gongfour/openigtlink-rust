//! Video Streaming Example
//!
//! This example demonstrates real-time video streaming using the OpenIGTLink
//! VIDEO message type with different codecs.
//!
//! # Usage
//!
//! ```bash
//! # Stream MJPEG video (640x480, 30fps, 10 seconds)
//! cargo run --example video_streaming mjpeg
//!
//! # Stream H.264 video (1920x1080, 60fps, 5 seconds)
//! cargo run --example video_streaming h264
//!
//! # Stream raw uncompressed video (320x240, 15fps, 10 seconds)
//! cargo run --example video_streaming raw
//! ```
//!
//! Make sure to run the server first:
//! ```bash
//! cargo run --example server
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::{ClientBuilder, SyncIgtlClient};
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{CodecType, VideoMessage};
use std::env;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let codec = parse_codec();

    // Connect to server
    let mut client = ClientBuilder::new().tcp("127.0.0.1:18944").sync().build()?;
    println!("[INFO] Connected to OpenIGTLink server\n");

    // Execute streaming scenario
    match codec {
        CodecType::MJPEG => stream_mjpeg(&mut client)?,
        CodecType::H264 => stream_h264(&mut client)?,
        CodecType::Raw => stream_raw(&mut client)?,
        _ => {
            eprintln!("Unsupported codec type");
            return Ok(());
        }
    }

    println!("\n[INFO] Video streaming completed successfully");
    Ok(())
}

fn parse_codec() -> CodecType {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].to_lowercase().as_str() {
            "mjpeg" => return CodecType::MJPEG,
            "h264" => return CodecType::H264,
            "raw" => return CodecType::Raw,
            _ => {}
        }
    }

    // Default codec
    println!("Usage: cargo run --example video_streaming [mjpeg|h264|raw]");
    println!("Defaulting to MJPEG...\n");
    CodecType::MJPEG
}

/// Stream MJPEG video (640x480, 30fps, 10 seconds)
///
/// MJPEG (Motion JPEG) is simple and has low latency, suitable for
/// surgical cameras and endoscopy where minimal delay is critical.
fn stream_mjpeg(client: &mut SyncIgtlClient) -> Result<()> {
    println!("=== MJPEG Video Streaming ===");
    println!("Resolution: 640x480");
    println!("Frame Rate: 30 fps");
    println!("Duration: 10 seconds");
    println!("Codec: Motion JPEG\n");

    let width = 640;
    let height = 480;
    let fps: usize = 30;
    let duration_sec = 10;
    let total_frames = fps * duration_sec;
    let frame_interval = Duration::from_millis(1000 / fps as u64);

    let mut actual_fps_sum = 0.0;
    let mut fps_samples = 0;

    for frame_num in 0..total_frames {
        let start_time = Instant::now();

        // Generate simulated MJPEG frame
        let frame_data = generate_mjpeg_frame(frame_num, width, height);

        // Create VIDEO message
        let video = VideoMessage::new(CodecType::MJPEG, width, height, frame_data);

        // Send to server
        let msg = IgtlMessage::new(video, "LaparoscopicCamera")?;
        client.send(&msg)?;

        // Display progress every second
        if frame_num % fps == 0 {
            let seconds = frame_num / fps;
            print!("\r[MJPEG] Streaming: {}/{} seconds", seconds, duration_sec);
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }

        // Calculate actual FPS
        let elapsed = start_time.elapsed();
        let actual_fps = 1.0 / elapsed.as_secs_f64();
        actual_fps_sum += actual_fps;
        fps_samples += 1;

        // Maintain frame rate
        if elapsed < frame_interval {
            thread::sleep(frame_interval - elapsed);
        }
    }

    println!(); // New line after progress
    println!("Average FPS: {:.2}", actual_fps_sum / fps_samples as f64);
    Ok(())
}

/// Stream H.264 video (1920x1080, 60fps, 5 seconds)
///
/// H.264 provides excellent compression for high-resolution video,
/// ideal for recording surgical procedures or telemedicine.
fn stream_h264(client: &mut SyncIgtlClient) -> Result<()> {
    println!("=== H.264 Video Streaming ===");
    println!("Resolution: 1920x1080 (Full HD)");
    println!("Frame Rate: 60 fps");
    println!("Duration: 5 seconds");
    println!("Codec: H.264/AVC\n");

    let width = 1920;
    let height = 1080;
    let fps: usize = 60;
    let duration_sec = 5;
    let total_frames = fps * duration_sec;
    let frame_interval = Duration::from_millis(1000 / fps as u64);

    let mut total_bytes = 0;

    for frame_num in 0..total_frames {
        let start_time = Instant::now();

        // Generate simulated H.264 frame
        let frame_data = generate_h264_frame(frame_num, width, height);
        total_bytes += frame_data.len();

        // Create VIDEO message
        let video = VideoMessage::new(CodecType::H264, width, height, frame_data);

        // Send to server
        let msg = IgtlMessage::new(video, "SurgicalMicroscope")?;
        client.send(&msg)?;

        // Display progress every second
        if frame_num % fps == 0 {
            let seconds = frame_num / fps;
            print!("\r[H.264] Streaming: {}/{} seconds", seconds, duration_sec);
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }

        // Maintain frame rate
        let elapsed = start_time.elapsed();
        if elapsed < frame_interval {
            thread::sleep(frame_interval - elapsed);
        }
    }

    println!(); // New line after progress
    println!(
        "Total data sent: {:.2} MB",
        total_bytes as f64 / 1_048_576.0
    );
    println!(
        "Average bitrate: {:.2} Mbps",
        (total_bytes as f64 * 8.0) / (duration_sec as f64 * 1_000_000.0)
    );
    Ok(())
}

/// Stream raw uncompressed video (320x240, 15fps, 10 seconds)
///
/// Raw video is uncompressed, useful for debugging or when
/// compression artifacts must be avoided.
fn stream_raw(client: &mut SyncIgtlClient) -> Result<()> {
    println!("=== Raw Uncompressed Video Streaming ===");
    println!("Resolution: 320x240");
    println!("Frame Rate: 15 fps");
    println!("Duration: 10 seconds");
    println!("Format: RGB24 (uncompressed)\n");

    let width = 320;
    let height = 240;
    let fps: usize = 15;
    let duration_sec = 10;
    let total_frames = fps * duration_sec;
    let frame_interval = Duration::from_millis(1000 / fps as u64);

    let mut total_bytes = 0;

    for frame_num in 0..total_frames {
        let start_time = Instant::now();

        // Generate simulated raw RGB frame
        let frame_data = generate_raw_frame(frame_num, width, height);
        total_bytes += frame_data.len();

        // Create VIDEO message
        let video = VideoMessage::new(CodecType::Raw, width, height, frame_data);

        // Send to server
        let msg = IgtlMessage::new(video, "MonitorCamera")?;
        client.send(&msg)?;

        // Display progress
        print!("\r[RAW] Streaming frame {}/{}", frame_num + 1, total_frames);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Maintain frame rate
        let elapsed = start_time.elapsed();
        if elapsed < frame_interval {
            thread::sleep(frame_interval - elapsed);
        }
    }

    println!(); // New line after progress
    println!(
        "Total data sent: {:.2} MB",
        total_bytes as f64 / 1_048_576.0
    );
    Ok(())
}

/// Generate simulated MJPEG frame
///
/// Creates a minimal JPEG structure with varying content
fn generate_mjpeg_frame(frame_num: usize, width: u16, height: u16) -> Vec<u8> {
    // Simulated JPEG frame (simplified structure)
    let mut data = Vec::new();

    // JPEG SOI (Start of Image)
    data.extend_from_slice(&[0xFF, 0xD8]);

    // Simulated JPEG data based on frame number
    let content_size = (width as usize * height as usize) / 20; // Compressed size
    for i in 0..content_size {
        let byte = ((frame_num + i) % 256) as u8;
        data.push(byte);
    }

    // JPEG EOI (End of Image)
    data.extend_from_slice(&[0xFF, 0xD9]);

    data
}

/// Generate simulated H.264 frame
///
/// Creates a minimal H.264 NAL unit structure
fn generate_h264_frame(frame_num: usize, width: u16, height: u16) -> Vec<u8> {
    let mut data = Vec::new();

    // H.264 NAL unit start code
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);

    // NAL unit header (simplified)
    let is_keyframe = frame_num % 30 == 0; // Keyframe every 30 frames
    let nal_type = if is_keyframe { 0x65 } else { 0x41 }; // IDR or non-IDR
    data.push(nal_type);

    // Simulated compressed data
    // H.264 compression is ~100:1 for typical video
    let content_size = (width as usize * height as usize * 3) / 100;
    for i in 0..content_size {
        let byte = ((frame_num * 7 + i * 13) % 256) as u8;
        data.push(byte);
    }

    data
}

/// Generate simulated raw RGB frame
///
/// Creates uncompressed RGB24 pixel data
fn generate_raw_frame(frame_num: usize, width: u16, height: u16) -> Vec<u8> {
    let mut data = Vec::with_capacity((width as usize * height as usize * 3) as usize);

    for y in 0..height {
        for x in 0..width {
            // Create animated gradient pattern
            let phase = frame_num as f32 * 0.1;
            let nx = x as f32 / width as f32;
            let ny = y as f32 / height as f32;

            // RGB channels with animation
            let r = ((nx * 255.0 + phase * 10.0).sin() * 127.0 + 128.0) as u8;
            let g = ((ny * 255.0 + phase * 15.0).cos() * 127.0 + 128.0) as u8;
            let b = (((nx + ny) * 255.0 + phase * 20.0).sin() * 127.0 + 128.0) as u8;

            data.push(r);
            data.push(g);
            data.push(b);
        }
    }

    data
}

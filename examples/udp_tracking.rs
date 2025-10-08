//! UDP High-Speed Tracking Example
//!
//! Demonstrates low-latency surgical tool tracking using UDP transport.
//! Compares TCP vs UDP performance for real-time position updates.
//!
//! # Usage
//!
//! ```bash
//! # Send tracking data at 120Hz via UDP
//! cargo run --example udp_tracking udp
//!
//! # Compare TCP vs UDP latency
//! cargo run --example udp_tracking compare
//!
//! # Run with custom parameters
//! cargo run --example udp_tracking custom 200 5
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::{builder::ClientBuilder, UdpClient};
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::TransformMessage;
use std::env;
use std::f32::consts::PI;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mode = env::args().nth(1).unwrap_or_else(|| "udp".to_string());

    match mode.as_str() {
        "udp" => test_udp_tracking()?,
        "compare" => compare_tcp_udp()?,
        "custom" => {
            let fps: usize = env::args()
                .nth(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(120);
            let duration_sec: usize = env::args()
                .nth(3)
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);
            test_custom_tracking(fps, duration_sec)?;
        }
        _ => {
            println!("Usage: cargo run --example udp_tracking [MODE]");
            println!("\nModes:");
            println!("  udp           - High-speed UDP tracking at 120Hz (default)");
            println!("  compare       - Compare TCP vs UDP latency");
            println!("  custom FPS DUR - Custom frequency and duration");
            println!("\nExample:");
            println!("  cargo run --example udp_tracking custom 200 5");
        }
    }

    Ok(())
}

/// Test UDP high-speed tracking at 120Hz
fn test_udp_tracking() -> Result<()> {
    println!("=== UDP High-Speed Tracking Test ===\n");
    println!("[INFO] Configuration:");
    println!("  Protocol: UDP (connectionless)");
    println!("  Update Rate: 120 Hz");
    println!("  Duration: 10 seconds");
    println!("  Total Frames: 1200");
    println!("  Motion: Circular path (100mm radius)\n");

    let client = UdpClient::bind("0.0.0.0:0")?;
    let target = "127.0.0.1:18944";

    println!("[INFO] Client bound to: {}", client.local_addr()?);
    println!("[INFO] Sending to: {}\n", target);

    let fps: usize = 120;
    let frame_duration = Duration::from_micros(1_000_000 / fps as u64);
    let total_frames = fps * 10; // 10 seconds

    let mut sent_count = 0;
    let mut total_latency = Duration::ZERO;
    let start_time = Instant::now();

    println!("[PROGRESS] Sending tracking data...\n");

    for frame in 0..total_frames {
        let frame_start = Instant::now();

        // Simulate circular motion
        let transform = simulate_tracker_position(frame, fps);
        let msg = IgtlMessage::new(transform, "UdpTracker")?;

        // Send via UDP
        client.send_to(&msg, target)?;
        sent_count += 1;

        let send_latency = frame_start.elapsed();
        total_latency += send_latency;

        // Progress indicator every second
        if frame % fps == 0 && frame > 0 {
            let seconds = frame / fps;
            let avg_latency_us = total_latency.as_micros() / sent_count as u128;
            println!(
                "  {:2}s: Frame {:4} | Avg latency: {:3} μs | Actual rate: {:.1} Hz",
                seconds,
                frame,
                avg_latency_us,
                sent_count as f64 / start_time.elapsed().as_secs_f64()
            );
        }

        // Maintain precise timing
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }

    let total_time = start_time.elapsed();
    let actual_fps = sent_count as f64 / total_time.as_secs_f64();

    println!("\n[RESULTS]");
    println!("  Frames sent: {}", sent_count);
    println!("  Total time: {:.2}s", total_time.as_secs_f64());
    println!("  Actual rate: {:.2} Hz", actual_fps);
    println!(
        "  Average latency: {:.1} μs",
        total_latency.as_micros() / sent_count as u128
    );
    println!("  Max theoretical rate: {:.0} Hz", 1_000_000.0 / (total_latency.as_micros() as f64 / sent_count as f64));

    Ok(())
}

/// Compare TCP vs UDP latency
fn compare_tcp_udp() -> Result<()> {
    println!("=== TCP vs UDP Latency Comparison ===\n");

    let test_frames = 100;
    let target = "127.0.0.1:18944";

    // Test UDP latency
    println!("[TEST 1] UDP Latency Measurement");
    println!("  Sending {} frames...", test_frames);

    let udp_client = UdpClient::bind("0.0.0.0:0")?;
    let mut udp_latencies = Vec::new();

    for frame in 0..test_frames {
        let start = Instant::now();
        let transform = simulate_tracker_position(frame, 100);
        let msg = IgtlMessage::new(transform, "UdpBench")?;
        udp_client.send_to(&msg, target)?;
        udp_latencies.push(start.elapsed());
    }

    let udp_avg = udp_latencies.iter().sum::<Duration>() / test_frames as u32;
    let udp_min = udp_latencies.iter().min().unwrap();
    let udp_max = udp_latencies.iter().max().unwrap();

    println!("  ✓ Complete");
    println!("    Average: {:.1} μs", udp_avg.as_micros());
    println!("    Min: {:.1} μs", udp_min.as_micros());
    println!("    Max: {:.1} μs", udp_max.as_micros());

    // Test TCP latency
    println!("\n[TEST 2] TCP Latency Measurement");
    println!("  Connecting to server...");

    let tcp_result = ClientBuilder::new().tcp(target).sync().build();

    if let Ok(mut tcp_client) = tcp_result {
        println!("  ✓ Connected");
        println!("  Sending {} frames...", test_frames);

        let mut tcp_latencies = Vec::new();

        for frame in 0..test_frames {
            let start = Instant::now();
            let transform = simulate_tracker_position(frame, 100);
            let msg = IgtlMessage::new(transform, "TcpBench")?;
            tcp_client.send(&msg)?;
            tcp_latencies.push(start.elapsed());
        }

        let tcp_avg = tcp_latencies.iter().sum::<Duration>() / test_frames as u32;
        let tcp_min = tcp_latencies.iter().min().unwrap();
        let tcp_max = tcp_latencies.iter().max().unwrap();

        println!("  ✓ Complete");
        println!("    Average: {:.1} μs", tcp_avg.as_micros());
        println!("    Min: {:.1} μs", tcp_min.as_micros());
        println!("    Max: {:.1} μs", tcp_max.as_micros());

        // Comparison
        println!("\n[COMPARISON]");
        println!("  ┌──────────────┬──────────────┬──────────────┬──────────────┐");
        println!("  │ Protocol     │ Avg (μs)     │ Min (μs)     │ Max (μs)     │");
        println!("  ├──────────────┼──────────────┼──────────────┼──────────────┤");
        println!(
            "  │ UDP          │ {:12.1} │ {:12.1} │ {:12.1} │",
            udp_avg.as_micros(),
            udp_min.as_micros(),
            udp_max.as_micros()
        );
        println!(
            "  │ TCP          │ {:12.1} │ {:12.1} │ {:12.1} │",
            tcp_avg.as_micros(),
            tcp_min.as_micros(),
            tcp_max.as_micros()
        );
        println!("  └──────────────┴──────────────┴──────────────┴──────────────┘");

        let improvement = ((tcp_avg.as_micros() as f64 - udp_avg.as_micros() as f64)
            / tcp_avg.as_micros() as f64)
            * 100.0;

        println!("\n  UDP is {:.1}% faster than TCP (average latency)", improvement);
        println!("  UDP max throughput: ~{:.0} Hz", 1_000_000.0 / udp_avg.as_micros() as f64);
        println!("  TCP max throughput: ~{:.0} Hz", 1_000_000.0 / tcp_avg.as_micros() as f64);
    } else {
        println!("  ✗ Failed to connect (server not running?)");
        println!("  Skipping TCP benchmark");
    }

    Ok(())
}

/// Custom tracking test with user-specified parameters
fn test_custom_tracking(fps: usize, duration_sec: usize) -> Result<()> {
    println!("=== Custom UDP Tracking Test ===\n");
    println!("[INFO] Configuration:");
    println!("  Update Rate: {} Hz", fps);
    println!("  Duration: {} seconds", duration_sec);
    println!("  Total Frames: {}\n", fps * duration_sec);

    let client = UdpClient::bind("0.0.0.0:0")?;
    let target = "127.0.0.1:18944";

    let frame_duration = Duration::from_micros(1_000_000 / fps as u64);
    let total_frames = fps * duration_sec;

    let start_time = Instant::now();

    for frame in 0..total_frames {
        let frame_start = Instant::now();

        let transform = simulate_tracker_position(frame, fps);
        let msg = IgtlMessage::new(transform, "CustomTracker")?;
        client.send_to(&msg, target)?;

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }

    let total_time = start_time.elapsed();
    let actual_fps = total_frames as f64 / total_time.as_secs_f64();

    println!("[RESULTS]");
    println!("  Frames sent: {}", total_frames);
    println!("  Actual rate: {:.2} Hz (target: {} Hz)", actual_fps, fps);
    println!("  Timing accuracy: {:.2}%", (actual_fps / fps as f64) * 100.0);

    Ok(())
}

/// Simulate tracker position with circular motion
///
/// # Arguments
/// * `frame` - Current frame number
/// * `fps` - Frames per second (affects motion speed)
fn simulate_tracker_position(frame: usize, fps: usize) -> TransformMessage {
    let t = frame as f32 / fps as f32;

    // Circular motion: 100mm radius, 1 revolution per 5 seconds
    let angle = (t / 5.0) * 2.0 * PI;
    let x = 100.0 * angle.cos();
    let y = 100.0 * angle.sin();
    let z = 50.0 + 10.0 * (t * 2.0).sin(); // Slight vertical oscillation

    // Create transformation with rotation around Z-axis
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    TransformMessage {
        matrix: [
            [cos_a, -sin_a, 0.0, x],
            [sin_a, cos_a, 0.0, y],
            [0.0, 0.0, 1.0, z],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_position() {
        let transform = simulate_tracker_position(0, 120);
        // At frame 0, should be at (100, 0, ~50)
        // Position is in matrix[i][3] (translation column)
        assert!((transform.matrix[0][3] - 100.0).abs() < 1.0);
        assert!(transform.matrix[1][3].abs() < 1.0);
        assert!((transform.matrix[2][3] - 50.0).abs() < 15.0);
    }

    #[test]
    fn test_circular_motion() {
        let fps = 120;
        let frames_per_revolution = fps * 5; // 5 seconds per revolution

        // Start position
        let start = simulate_tracker_position(0, fps);

        // Quarter revolution
        let quarter = simulate_tracker_position(frames_per_revolution / 4, fps);

        // Should rotate 90 degrees: from (100, 0) to (0, 100)
        assert!((start.matrix[0][3] - 100.0).abs() < 1.0);
        assert!((quarter.matrix[1][3] - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_frame_duration_120hz() {
        let fps: usize = 120;
        let duration = Duration::from_micros(1_000_000 / fps as u64);
        assert_eq!(duration.as_micros(), 8333); // 1/120 sec ≈ 8333 μs
    }

    #[test]
    fn test_frame_duration_200hz() {
        let fps: usize = 200;
        let duration = Duration::from_micros(1_000_000 / fps as u64);
        assert_eq!(duration.as_micros(), 5000); // 1/200 sec = 5000 μs
    }
}

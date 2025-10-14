//! Surgical Tool Tracking Server Example
//!
//! This example demonstrates real-time tracking of multiple surgical instruments
//! using the OpenIGTLink TDATA (Tracking Data) message type.
//!
//! # Usage
//!
//! ```bash
//! # Track 3 surgical tools (scalpel, probe, catheter) at 60Hz for 10 seconds
//! cargo run --example tracking_server
//! ```
//!
//! Make sure to run the server first:
//! ```bash
//! cargo run --example server
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::{ClientBuilder, SyncIgtlClient};
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{
    TDataMessage, TrackingDataElement, TrackingInstrumentType,
};
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
    // Connect to server
    let mut client = ClientBuilder::new().tcp("127.0.0.1:18944").sync().build()?;
    println!("[INFO] Connected to OpenIGTLink server\n");

    println!("=== Surgical Tool Tracking Simulation ===");
    println!("Instruments: Scalpel, Probe, Catheter");
    println!("Update Rate: 60 Hz");
    println!("Duration: 10 seconds");
    println!("Tracking System: Optical (simulated)\n");

    simulate_tracking(&mut client)?;

    println!("\n[INFO] Tracking simulation completed successfully");
    Ok(())
}

/// Simulate real-time tracking of 3 surgical instruments
///
/// Each instrument follows a different motion pattern:
/// - Scalpel: Circular cutting motion
/// - Probe: Palpation motion (up/down)
/// - Catheter: Linear insertion
fn simulate_tracking(client: &mut SyncIgtlClient) -> Result<()> {
    let update_rate: usize = 60; // Hz
    let duration_sec = 10;
    let total_frames = update_rate * duration_sec;
    let frame_interval = Duration::from_millis(1000 / update_rate as u64);

    for frame_num in 0..total_frames {
        let start_time = Instant::now();
        let time_sec = frame_num as f32 / update_rate as f32;

        // Create tracking data for all instruments
        let mut tdata = TDataMessage::empty();

        // Scalpel: Circular cutting motion (radius 30mm, centered at [100, 50, 200])
        let scalpel = create_scalpel_tracking(time_sec);
        tdata.add_element(scalpel);

        // Probe: Palpation motion (up/down 20mm, centered at [150, 80, 180])
        let probe = create_probe_tracking(time_sec);
        tdata.add_element(probe);

        // Catheter: Linear insertion (moving along Z-axis)
        let catheter = create_catheter_tracking(time_sec);
        tdata.add_element(catheter);

        // Send tracking data
        let msg = IgtlMessage::new(tdata.clone(), "OpticalTracker")?;
        client.send(&msg)?;

        // Display tracking info every second
        if frame_num % update_rate == 0 {
            let seconds = frame_num / update_rate;
            println!("[{}s] Tracking update:", seconds);
            for element in &tdata.elements {
                let x = element.matrix[0][3];
                let y = element.matrix[1][3];
                let z = element.matrix[2][3];
                println!(
                    "  {:<10} position: ({:7.2}, {:7.2}, {:7.2}) mm",
                    element.name, x, y, z
                );
            }
        }

        // Maintain update rate
        let elapsed = start_time.elapsed();
        if elapsed < frame_interval {
            thread::sleep(frame_interval - elapsed);
        }
    }

    Ok(())
}

/// Create tracking data for scalpel with circular cutting motion
///
/// Motion pattern: Circular path in XY plane
/// - Center: (100, 50, 200) mm
/// - Radius: 30 mm
/// - Period: 3 seconds per revolution
fn create_scalpel_tracking(time_sec: f32) -> TrackingDataElement {
    let center_x = 100.0;
    let center_y = 50.0;
    let center_z = 200.0;
    let radius = 30.0;
    let period = 3.0; // seconds

    let angle = (time_sec / period) * 2.0 * PI;
    let x = center_x + radius * angle.cos();
    let y = center_y + radius * angle.sin();
    let z = center_z;

    // Create rotation matrix to align tool with motion direction
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    TrackingDataElement {
        name: "Scalpel".to_string(),
        instrument_type: TrackingInstrumentType::Instrument6D,
        matrix: [
            [cos_a, -sin_a, 0.0, x],
            [sin_a, cos_a, 0.0, y],
            [0.0, 0.0, 1.0, z],
        ],
    }
}

/// Create tracking data for probe with palpation motion
///
/// Motion pattern: Vertical oscillation (palpating tissue)
/// - Center: (150, 80, 180) mm
/// - Amplitude: 20 mm (up/down)
/// - Frequency: 2 Hz
fn create_probe_tracking(time_sec: f32) -> TrackingDataElement {
    let center_x = 150.0;
    let center_y = 80.0;
    let center_z = 180.0;
    let amplitude = 20.0;
    let frequency = 2.0; // Hz

    let x = center_x;
    let y = center_y;
    let z = center_z + amplitude * (time_sec * frequency * 2.0 * PI).sin();

    TrackingDataElement::with_translation("Probe", TrackingInstrumentType::Instrument6D, x, y, z)
}

/// Create tracking data for catheter with linear insertion
///
/// Motion pattern: Linear insertion along Z-axis
/// - Start: (80, 120, 150) mm
/// - Speed: 10 mm/s
/// - Direction: +Z (insertion)
fn create_catheter_tracking(time_sec: f32) -> TrackingDataElement {
    let start_x = 80.0;
    let start_y = 120.0;
    let start_z = 150.0;
    let speed = 10.0; // mm/s

    let x = start_x;
    let y = start_y;
    let z = start_z + speed * time_sec;

    // Add slight rotation to simulate catheter flexibility
    let rotation_angle = time_sec * 0.1;
    let cos_r = rotation_angle.cos();
    let sin_r = rotation_angle.sin();

    TrackingDataElement {
        name: "Catheter".to_string(),
        instrument_type: TrackingInstrumentType::Instrument6D,
        matrix: [
            [cos_r, -sin_r, 0.0, x],
            [sin_r, cos_r, 0.0, y],
            [0.0, 0.0, 1.0, z],
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalpel_tracking() {
        // Test that scalpel follows circular path
        let t0 = create_scalpel_tracking(0.0);
        let t1 = create_scalpel_tracking(0.75); // Quarter period (3s/4)

        // At t=0, should be at starting angle
        let x0 = t0.matrix[0][3];
        let y0 = t0.matrix[1][3];

        // At t=0.75s, should have rotated 90 degrees
        let x1 = t1.matrix[0][3];
        let y1 = t1.matrix[1][3];

        // X and Y should be different (circular motion)
        assert!((x0 - x1).abs() > 10.0 || (y0 - y1).abs() > 10.0);
    }

    #[test]
    fn test_probe_tracking() {
        // Test that probe oscillates vertically
        let t0 = create_probe_tracking(0.0);
        let t1 = create_probe_tracking(0.125); // Quarter period (1/(2*2Hz) = 0.125s)

        let z0 = t0.matrix[2][3];
        let z1 = t1.matrix[2][3];

        // Z should change (vertical oscillation)
        // At 2Hz with 20mm amplitude, max displacement is 20mm
        assert!((z0 - z1).abs() > 10.0);

        // X and Y should remain constant
        assert_eq!(t0.matrix[0][3], t1.matrix[0][3]);
        assert_eq!(t0.matrix[1][3], t1.matrix[1][3]);
    }

    #[test]
    fn test_catheter_tracking() {
        // Test that catheter moves linearly along Z
        let t0 = create_catheter_tracking(0.0);
        let t1 = create_catheter_tracking(1.0); // 1 second later

        let z0 = t0.matrix[2][3];
        let z1 = t1.matrix[2][3];

        // Z should increase by speed (10 mm/s)
        let expected_delta = 10.0;
        assert!((z1 - z0 - expected_delta).abs() < 0.1);
    }

    #[test]
    fn test_tdata_message_creation() {
        // Test creating TDATA message with multiple elements
        let mut tdata = TDataMessage::empty();
        assert_eq!(tdata.len(), 0);

        tdata.add_element(create_scalpel_tracking(0.0));
        tdata.add_element(create_probe_tracking(0.0));
        tdata.add_element(create_catheter_tracking(0.0));

        assert_eq!(tdata.len(), 3);
        assert_eq!(tdata.elements[0].name, "Scalpel");
        assert_eq!(tdata.elements[1].name, "Probe");
        assert_eq!(tdata.elements[2].name, "Catheter");
    }
}

//! Sensor Data Logger Example
//!
//! This example demonstrates real-time sensor data collection and transmission
//! using the OpenIGTLink SENSOR message type.
//!
//! # Usage
//!
//! ```bash
//! # Log force sensor data (6-axis F/T sensor, 100Hz, 10 seconds)
//! cargo run --example sensor_logger force
//!
//! # Log IMU sensor data (accelerometer + gyroscope, 100Hz, 10 seconds)
//! cargo run --example sensor_logger imu
//!
//! # Log combined sensor array (8 force + 6 IMU = 14 channels, 100Hz, 10 seconds)
//! cargo run --example sensor_logger combined
//! ```
//!
//! Make sure to run the server first:
//! ```bash
//! cargo run --example server
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::{ClientBuilder, SyncIgtlClient};
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::SensorMessage;
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
    let sensor_type = parse_sensor_type();

    // Connect to server
    let mut client = ClientBuilder::new().tcp("127.0.0.1:18944").sync().build()?;
    println!("[INFO] Connected to OpenIGTLink server\n");

    // Execute sensor logging scenario
    match sensor_type.as_str() {
        "force" => log_force_sensor(&mut client)?,
        "imu" => log_imu_sensor(&mut client)?,
        "combined" => log_combined_sensors(&mut client)?,
        _ => unreachable!(),
    }

    println!("\n[INFO] Sensor data logging completed successfully");
    Ok(())
}

fn parse_sensor_type() -> String {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let sensor_type = args[1].to_lowercase();
        if ["force", "imu", "combined"].contains(&sensor_type.as_str()) {
            return sensor_type;
        }
    }

    // Default sensor type
    println!("Usage: cargo run --example sensor_logger [force|imu|combined]");
    println!("Defaulting to force sensor...\n");
    "force".to_string()
}

/// Log 6-axis force/torque sensor data (100Hz, 10 seconds)
///
/// Simulates an ATI Force/Torque sensor commonly used in robotic surgery
/// for haptic feedback.
fn log_force_sensor(client: &mut SyncIgtlClient) -> Result<()> {
    println!("=== Force/Torque Sensor Logging ===");
    println!("Channels: 6 (Fx, Fy, Fz, Tx, Ty, Tz)");
    println!("Sample Rate: 100 Hz");
    println!("Duration: 10 seconds");
    println!("Unit: Newton (force) + Newton-meter (torque)\n");

    let sample_rate: usize = 100;
    let duration_sec = 10;
    let total_samples = sample_rate * duration_sec;
    let sample_interval = Duration::from_millis(1000 / sample_rate as u64);

    for sample_num in 0..total_samples {
        let start_time = Instant::now();

        // Read simulated force/torque sensor
        let sensor_data = read_force_sensor(sample_num);

        // Create SENSOR message
        // Unit: 0x0101 = Newton (force) + Newton-meter (torque)
        let sensor = SensorMessage::with_unit(1, 0x0101, sensor_data.clone())?;

        // Send to server
        let msg = IgtlMessage::new(sensor, "ATI_ForceSensor")?;
        client.send(&msg)?;

        // Display readings every second
        if sample_num % sample_rate == 0 {
            let seconds = sample_num / sample_rate;
            println!(
                "[{}s] Force: [{:6.2}, {:6.2}, {:6.2}] N  Torque: [{:6.3}, {:6.3}, {:6.3}] Nm",
                seconds,
                sensor_data[0],
                sensor_data[1],
                sensor_data[2],
                sensor_data[3],
                sensor_data[4],
                sensor_data[5]
            );
        }

        // Maintain sample rate
        let elapsed = start_time.elapsed();
        if elapsed < sample_interval {
            thread::sleep(sample_interval - elapsed);
        }
    }

    Ok(())
}

/// Log IMU sensor data (100Hz, 10 seconds)
///
/// Simulates a 6-DOF IMU (3-axis accelerometer + 3-axis gyroscope)
/// used for surgical instrument orientation tracking.
fn log_imu_sensor(client: &mut SyncIgtlClient) -> Result<()> {
    println!("=== IMU Sensor Logging ===");
    println!("Channels: 6 (Accel X,Y,Z + Gyro X,Y,Z)");
    println!("Sample Rate: 100 Hz");
    println!("Duration: 10 seconds");
    println!("Unit: m/s² (accel) + rad/s (gyro)\n");

    let sample_rate: usize = 100;
    let duration_sec = 10;
    let total_samples = sample_rate * duration_sec;
    let sample_interval = Duration::from_millis(1000 / sample_rate as u64);

    for sample_num in 0..total_samples {
        let start_time = Instant::now();

        // Read simulated IMU sensor
        let sensor_data = read_imu_sensor(sample_num);

        // Create SENSOR message
        // Unit: 0x0202 = m/s² (accel) + rad/s (gyro)
        let sensor = SensorMessage::with_unit(1, 0x0202, sensor_data.clone())?;

        // Send to server
        let msg = IgtlMessage::new(sensor, "IMU_9DOF")?;
        client.send(&msg)?;

        // Display readings every second
        if sample_num % sample_rate == 0 {
            let seconds = sample_num / sample_rate;
            println!(
                "[{}s] Accel: [{:6.2}, {:6.2}, {:6.2}] m/s²  Gyro: [{:6.3}, {:6.3}, {:6.3}] rad/s",
                seconds,
                sensor_data[0],
                sensor_data[1],
                sensor_data[2],
                sensor_data[3],
                sensor_data[4],
                sensor_data[5]
            );
        }

        // Maintain sample rate
        let elapsed = start_time.elapsed();
        if elapsed < sample_interval {
            thread::sleep(sample_interval - elapsed);
        }
    }

    Ok(())
}

/// Log combined sensor array (100Hz, 10 seconds)
///
/// Simulates a multi-sensor system with 8 force channels + 6 IMU channels
/// (14 total channels) for comprehensive surgical instrument monitoring.
fn log_combined_sensors(client: &mut SyncIgtlClient) -> Result<()> {
    println!("=== Combined Sensor Array Logging ===");
    println!("Channels: 14 (8 force + 6 IMU)");
    println!("Sample Rate: 100 Hz");
    println!("Duration: 10 seconds\n");

    let sample_rate: usize = 100;
    let duration_sec = 10;
    let total_samples = sample_rate * duration_sec;
    let sample_interval = Duration::from_millis(1000 / sample_rate as u64);

    for sample_num in 0..total_samples {
        let start_time = Instant::now();

        // Read all sensors
        let sensor_data = read_combined_sensors(sample_num);

        // Create SENSOR message
        let sensor = SensorMessage::with_unit(1, 0, sensor_data.clone())?;

        // Send to server
        let msg = IgtlMessage::new(sensor, "SensorArray")?;
        client.send(&msg)?;

        // Display readings every second
        if sample_num % sample_rate == 0 {
            let seconds = sample_num / sample_rate;
            println!("[{}s] 14 channels: Force[0-7], IMU[8-13]", seconds);
            print!("      Force: ");
            for value in sensor_data.iter().take(8) {
                print!("{:6.2} ", value);
            }
            println!();
            print!("      IMU:   ");
            for value in sensor_data.iter().take(14).skip(8) {
                print!("{:6.2} ", value);
            }
            println!();
        }

        // Maintain sample rate
        let elapsed = start_time.elapsed();
        if elapsed < sample_interval {
            thread::sleep(sample_interval - elapsed);
        }
    }

    Ok(())
}

/// Read simulated 6-axis force/torque sensor
///
/// Returns [Fx, Fy, Fz, Tx, Ty, Tz] in Newton and Newton-meter
fn read_force_sensor(sample_num: usize) -> Vec<f64> {
    let t = sample_num as f64 * 0.01; // Time in seconds

    vec![
        2.5 * (t * 2.0).sin(),       // Fx (N)
        -1.2 * (t * 1.5).cos(),      // Fy (N)
        5.8 + 0.5 * (t * 3.0).sin(), // Fz (N) - mostly positive (pushing)
        0.15 * (t * 1.0).sin(),      // Tx (Nm)
        -0.08 * (t * 1.2).cos(),     // Ty (Nm)
        0.22 * (t * 0.8).sin(),      // Tz (Nm)
    ]
}

/// Read simulated IMU sensor (accelerometer + gyroscope)
///
/// Returns [Ax, Ay, Az, Gx, Gy, Gz] in m/s² and rad/s
fn read_imu_sensor(sample_num: usize) -> Vec<f64> {
    let t = sample_num as f64 * 0.01; // Time in seconds

    vec![
        0.5 * (t * 2.0).sin(),        // Ax (m/s²)
        -0.3 * (t * 1.8).cos(),       // Ay (m/s²)
        9.81 + 0.2 * (t * 2.5).sin(), // Az (m/s²) - gravity + motion
        0.1 * (t * 1.5).sin(),        // Gx (rad/s)
        -0.05 * (t * 1.2).cos(),      // Gy (rad/s)
        0.08 * (t * 2.0).sin(),       // Gz (rad/s)
    ]
}

/// Read simulated combined sensor array
///
/// Returns 14 channels: 8 force sensors + 6 IMU channels
fn read_combined_sensors(sample_num: usize) -> Vec<f64> {
    let t = sample_num as f64 * 0.01; // Time in seconds
    let mut data = Vec::with_capacity(14);

    // Force sensors (8 channels)
    for i in 0..8 {
        let value = (t * (1.0 + i as f64 * 0.2)).sin() * (1.0 + i as f64 * 0.5);
        data.push(value);
    }

    // IMU sensors (6 channels)
    // Accelerometer (3 channels)
    data.push(0.5 * (t * 2.0).sin());
    data.push(-0.3 * (t * 1.8).cos());
    data.push(9.81 + 0.2 * (t * 2.5).sin());

    // Gyroscope (3 channels)
    data.push(0.1 * (t * 1.5).sin());
    data.push(-0.05 * (t * 1.2).cos());
    data.push(0.08 * (t * 2.0).sin());

    data
}

//! NDARRAY Message - N-Dimensional Array Transfer Example
//!
//! This example demonstrates transferring multi-dimensional numerical arrays,
//! useful for matrix data, lookup tables, or volumetric datasets.
//!
//! # Usage
//!
//! ```bash
//! # Transfer 1D, 2D, and 3D arrays
//! cargo run --example ndarray_transfer
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::IgtlClient;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{NdArrayMessage, ScalarType};
use std::thread;
use std::time::Duration;

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    println!("=== NDARRAY Message: Multi-Dimensional Array Transfer ===\n");

    // Connect to server
    let mut client = IgtlClient::connect("127.0.0.1:18944")?;
    println!("[INFO] Connected to OpenIGTLink server\n");

    // Scenario 1: 1D array (signal/waveform)
    println!("[SCENARIO 1] 1D Array - ECG Waveform");
    println!("  Transferring cardiac signal data...\n");
    send_1d_waveform(&mut client)?;
    thread::sleep(Duration::from_secs(1));

    // Scenario 2: 2D array (transformation matrix)
    println!("\n[SCENARIO 2] 2D Array - Transformation Matrix");
    println!("  Transferring 4x4 homogeneous transform...\n");
    send_2d_matrix(&mut client)?;
    thread::sleep(Duration::from_secs(1));

    // Scenario 3: 3D array (volume data)
    println!("\n[SCENARIO 3] 3D Array - Lookup Table Volume");
    println!("  Transferring color mapping table...\n");
    send_3d_lut(&mut client)?;

    println!("\n[INFO] All arrays transferred successfully");
    Ok(())
}

/// Send 1D array: Simulated ECG waveform (Float32)
fn send_1d_waveform(client: &mut IgtlClient) -> Result<()> {
    // Generate 100-sample ECG-like waveform
    let sample_count = 100;
    let mut data = Vec::new();

    for i in 0..sample_count {
        let t = i as f32 / sample_count as f32;
        // Simplified ECG waveform (P-QRS-T complex)
        let value = if t < 0.2 {
            0.3 * (t * 5.0 * std::f32::consts::PI).sin() // P wave
        } else if t < 0.4 {
            -0.2 // PQ segment
        } else if t < 0.5 {
            5.0 * ((t - 0.4) * 10.0 * std::f32::consts::PI).sin() // QRS complex
        } else if t < 0.7 {
            0.0 // ST segment
        } else if t < 0.9 {
            0.5 * ((t - 0.7) * 5.0 * std::f32::consts::PI).sin() // T wave
        } else {
            0.0 // Baseline
        };

        data.extend_from_slice(&value.to_be_bytes());
    }

    let array = NdArrayMessage::new_1d(ScalarType::Float32, data)?;

    println!("  Array Properties:");
    println!("    Dimensions: 1D");
    println!("    Size: [{}]", sample_count);
    println!("    Data Type: Float32");
    println!("    Total Elements: {}", array.element_count());
    println!("    Data Size: {} bytes", array.data_size());

    let msg = IgtlMessage::new(array, "ECG_Waveform")?;
    client.send(&msg)?;

    println!("    ✓ Sent");

    Ok(())
}

/// Send 2D array: 4x4 transformation matrix (Float64)
fn send_2d_matrix(client: &mut IgtlClient) -> Result<()> {
    // Create 4x4 identity matrix with translation
    #[rustfmt::skip]
    let matrix: [f64; 16] = [
        1.0, 0.0, 0.0, 10.0,  // Row 0: X-axis + translation
        0.0, 1.0, 0.0, 20.0,  // Row 1: Y-axis + translation
        0.0, 0.0, 1.0, 30.0,  // Row 2: Z-axis + translation
        0.0, 0.0, 0.0, 1.0,   // Row 3: Homogeneous
    ];

    let mut data = Vec::new();
    for &value in &matrix {
        data.extend_from_slice(&value.to_be_bytes());
    }

    let array = NdArrayMessage::new_2d(ScalarType::Float64, 4, 4, data)?;

    println!("  Array Properties:");
    println!("    Dimensions: 2D");
    println!("    Size: [4, 4]");
    println!("    Data Type: Float64");
    println!("    Total Elements: {}", array.element_count());
    println!("    Data Size: {} bytes", array.data_size());
    println!("    Represents: Translation (10, 20, 30) mm");

    let msg = IgtlMessage::new(array, "TransformMatrix")?;
    client.send(&msg)?;

    println!("    ✓ Sent");

    Ok(())
}

/// Send 3D array: RGB lookup table (Uint8)
fn send_3d_lut(client: &mut IgtlClient) -> Result<()> {
    // Create 16x16x3 RGB color lookup table
    let lut_size = 16;
    let mut data = Vec::new();

    // Generate gradient: Red->Green->Blue
    for r in 0..lut_size {
        for g in 0..lut_size {
            // R channel: decreases with r
            let red = ((lut_size - r - 1) * 255 / (lut_size - 1)) as u8;

            // G channel: increases with g
            let green = (g * 255 / (lut_size - 1)) as u8;

            // B channel: constant gradient
            let blue = ((r + g) * 255 / (2 * (lut_size - 1))) as u8;

            data.push(red);
            data.push(green);
            data.push(blue);
        }
    }

    let array = NdArrayMessage::new_3d(ScalarType::Uint8, lut_size as u16, lut_size as u16, 3, data)?;

    println!("  Array Properties:");
    println!("    Dimensions: 3D");
    println!("    Size: [{}, {}, 3] (height, width, RGB)", lut_size, lut_size);
    println!("    Data Type: Uint8");
    println!("    Total Elements: {}", array.element_count());
    println!("    Data Size: {} bytes", array.data_size());
    println!("    Represents: RGB color lookup table");

    let msg = IgtlMessage::new(array, "ColorLUT")?;
    client.send(&msg)?;

    println!("    ✓ Sent");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1d_array_creation() {
        let data: Vec<u8> = vec![1, 2, 3, 4];
        let array = NdArrayMessage::new_1d(ScalarType::Uint8, data).unwrap();

        assert_eq!(array.ndim(), 1);
        assert_eq!(array.element_count(), 4);
    }

    #[test]
    fn test_2d_array_creation() {
        // 4x4 uint8 = 16 bytes
        let data = vec![0u8; 16];
        let array = NdArrayMessage::new_2d(ScalarType::Uint8, 4, 4, data).unwrap();

        assert_eq!(array.ndim(), 2);
        assert_eq!(array.size, vec![4, 4]);
        assert_eq!(array.element_count(), 16);
    }

    #[test]
    fn test_3d_array_creation() {
        // 16x16x3 uint8 = 768 bytes
        let data = vec![0u8; 768];
        let array = NdArrayMessage::new_3d(ScalarType::Uint8, 16, 16, 3, data).unwrap();

        assert_eq!(array.ndim(), 3);
        assert_eq!(array.size, vec![16, 16, 3]);
        assert_eq!(array.element_count(), 768);
    }

    #[test]
    fn test_float32_encoding() {
        let value = 3.14159f32;
        let mut data = Vec::new();
        data.extend_from_slice(&value.to_be_bytes());

        let array = NdArrayMessage::new_1d(ScalarType::Float32, data.clone()).unwrap();
        assert_eq!(array.data, data);
    }

    #[test]
    fn test_scalar_type_sizes() {
        assert_eq!(ScalarType::Uint8.size(), 1);
        assert_eq!(ScalarType::Float32.size(), 4);
        assert_eq!(ScalarType::Float64.size(), 8);
    }
}

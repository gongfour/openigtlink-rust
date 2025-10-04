//! Surgical Navigation - Fiducial Point Registration Example
//!
//! This example demonstrates patient-to-image registration using anatomical
//! fiducial points for surgical navigation systems.
//!
//! # Usage
//!
//! ```bash
//! # Register 5 anatomical fiducial points
//! cargo run --example point_navigation
//! ```
//!
//! Make sure to run the server first:
//! ```bash
//! cargo run --example server
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::IgtlClient;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{PointElement, PointMessage};

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // Connect to server
    let mut client = IgtlClient::connect("127.0.0.1:18944")?;
    println!("[INFO] Connected to OpenIGTLink server\n");

    println!("=== Surgical Navigation: Fiducial Point Registration ===");
    println!("System: Image-Guided Surgery Navigation");
    println!("Image Modality: CT scan (pre-operative)");
    println!("Coordinate System: LPS (Left-Posterior-Superior)\n");

    register_fiducials(&mut client)?;

    println!("\n[INFO] Fiducial registration completed successfully");
    Ok(())
}

/// Register anatomical fiducial points for patient-to-image registration
///
/// In surgical navigation, fiducial points are used to align the patient's
/// physical position with pre-operative CT/MRI images. These points are:
/// 1. Identified on pre-operative images (image space)
/// 2. Located on the patient during surgery (physical space)
/// 3. Used to compute transformation matrix for registration
///
/// # Fiducial Point Selection Guidelines
///
/// - Choose bony landmarks (stable, easy to identify)
/// - Distribute points across surgical field (avoid coplanar)
/// - Minimum 3 points required (4-6 points recommended)
/// - Include points near surgical target for accuracy
fn register_fiducials(client: &mut IgtlClient) -> Result<()> {
    println!("[STEP 1] Defining anatomical fiducial points from CT image...\n");

    // Define anatomical fiducial points
    // Coordinates in millimeters (LPS coordinate system)
    let fiducials = vec![
        // Fiducial 1: Nasion (bridge of nose)
        create_fiducial(
            "Nasion",
            "Fiducials",
            [0.0, 85.0, -40.0],
            [255, 0, 0, 255], // Red
            5.0,
            "CTImage",
        ),
        // Fiducial 2: Left preauricular point (in front of left ear)
        create_fiducial(
            "LeftEar",
            "Fiducials",
            [-70.0, 0.0, -60.0],
            [0, 255, 0, 255], // Green
            5.0,
            "CTImage",
        ),
        // Fiducial 3: Right preauricular point (in front of right ear)
        create_fiducial(
            "RightEar",
            "Fiducials",
            [70.0, 0.0, -60.0],
            [0, 0, 255, 255], // Blue
            5.0,
            "CTImage",
        ),
        // Fiducial 4: Target lesion (tumor center)
        create_fiducial(
            "TargetLesion",
            "Targets",
            [15.0, 30.0, 45.0],
            [255, 255, 0, 255], // Yellow
            8.0,
            "CTImage",
        ),
        // Fiducial 5: Entry point (planned trajectory start)
        create_fiducial(
            "EntryPoint",
            "Targets",
            [10.0, 25.0, 30.0],
            [255, 0, 255, 255], // Magenta
            4.0,
            "CTImage",
        ),
    ];

    println!("  Registering {} fiducial points:", fiducials.len());
    println!("  ┌─────────────────┬──────────────────────────────────┐");
    println!("  │ Name            │ Position (LPS, mm)               │");
    println!("  ├─────────────────┼──────────────────────────────────┤");

    for fid in &fiducials {
        println!(
            "  │ {:<15} │ ({:7.1}, {:7.1}, {:7.1})          │",
            fid.name, fid.position[0], fid.position[1], fid.position[2]
        );
    }

    println!("  └─────────────────┴──────────────────────────────────┘\n");

    println!("[STEP 2] Sending fiducial data to navigation system...\n");

    // Create POINT message
    let point_msg = PointMessage::new(fiducials);

    // Send message
    let msg = IgtlMessage::new(point_msg.clone(), "NavigationSystem")?;
    client.send(&msg)?;

    println!("  ✓ {} points transmitted successfully", point_msg.len());
    println!("  ✓ Total data size: {} bytes", point_msg.len() * 136);

    println!("\n[STEP 3] Registration workflow:\n");
    println!("  1. Identify corresponding points on physical patient");
    println!("  2. Use pointer/probe to digitize points in physical space");
    println!("  3. Compute transformation matrix (image → physical space)");
    println!("  4. Calculate registration error (RMS of fiducial distances)");
    println!("  5. Verify registration accuracy < 2mm for neuro procedures");

    Ok(())
}

/// Create a fiducial point element with full metadata
///
/// # Arguments
/// * `name` - Point identifier (e.g., "Nasion", "LeftEar")
/// * `group` - Logical grouping (e.g., "Fiducials", "Targets")
/// * `position` - 3D coordinates [x, y, z] in millimeters (LPS)
/// * `rgba` - Color for visualization [R, G, B, A] (0-255)
/// * `diameter` - Sphere diameter for rendering (mm)
/// * `owner` - Associated image/coordinate frame
fn create_fiducial(
    name: &str,
    group: &str,
    position: [f32; 3],
    rgba: [u8; 4],
    diameter: f32,
    owner: &str,
) -> PointElement {
    PointElement::with_details(name, group, rgba, position, diameter, owner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use openigtlink_rust::protocol::message::Message;

    #[test]
    fn test_create_fiducial() {
        let fid = create_fiducial(
            "TestPoint",
            "TestGroup",
            [10.0, 20.0, 30.0],
            [255, 0, 0, 255],
            5.0,
            "TestImage",
        );

        assert_eq!(fid.name, "TestPoint");
        assert_eq!(fid.group, "TestGroup");
        assert_eq!(fid.position, [10.0, 20.0, 30.0]);
        assert_eq!(fid.rgba, [255, 0, 0, 255]);
        assert_eq!(fid.diameter, 5.0);
        assert_eq!(fid.owner, "TestImage");
    }

    #[test]
    fn test_fiducial_count() {
        // Test that we're registering correct number of points
        let fiducials = vec![
            create_fiducial("P1", "G", [0.0, 0.0, 0.0], [255, 0, 0, 255], 5.0, "I"),
            create_fiducial("P2", "G", [1.0, 1.0, 1.0], [0, 255, 0, 255], 5.0, "I"),
            create_fiducial("P3", "G", [2.0, 2.0, 2.0], [0, 0, 255, 255], 5.0, "I"),
        ];

        let msg = PointMessage::new(fiducials);
        assert_eq!(msg.len(), 3);
    }

    #[test]
    fn test_point_message_encoding() {
        let fid = create_fiducial(
            "Nasion",
            "Fiducials",
            [0.0, 85.0, -40.0],
            [255, 0, 0, 255],
            5.0,
            "CTImage",
        );

        let msg = PointMessage::new(vec![fid]);
        let encoded = msg.encode_content().unwrap();

        // Each point element is 136 bytes
        assert_eq!(encoded.len(), 136);
    }

    #[test]
    fn test_fiducial_colors() {
        // Test distinct colors for visualization
        let colors = vec![
            [255, 0, 0, 255],   // Red
            [0, 255, 0, 255],   // Green
            [0, 0, 255, 255],   // Blue
            [255, 255, 0, 255], // Yellow
            [255, 0, 255, 255], // Magenta
        ];

        for (i, color) in colors.iter().enumerate() {
            let fid = create_fiducial(
                &format!("Point{}", i + 1),
                "Test",
                [0.0, 0.0, 0.0],
                *color,
                5.0,
                "Test",
            );

            assert_eq!(fid.rgba, *color);
        }
    }

    #[test]
    fn test_lps_coordinates() {
        // Test that fiducials use LPS coordinate system conventions
        // Left ear: negative X (left)
        let left_ear = create_fiducial(
            "LeftEar",
            "Fiducials",
            [-70.0, 0.0, -60.0],
            [0, 255, 0, 255],
            5.0,
            "CT",
        );
        assert!(left_ear.position[0] < 0.0); // Negative X = Left

        // Right ear: positive X (right)
        let right_ear = create_fiducial(
            "RightEar",
            "Fiducials",
            [70.0, 0.0, -60.0],
            [0, 0, 255, 255],
            5.0,
            "CT",
        );
        assert!(right_ear.position[0] > 0.0); // Positive X = Right

        // Nasion: anterior (positive Y in LPS)
        let nasion = create_fiducial(
            "Nasion",
            "Fiducials",
            [0.0, 85.0, -40.0],
            [255, 0, 0, 255],
            5.0,
            "CT",
        );
        assert!(nasion.position[1] > 0.0); // Positive Y = Posterior
    }
}

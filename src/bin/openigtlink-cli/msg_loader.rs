use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::*;
use openigtlink_rust::protocol::AnyMessage;
use openigtlink_rust::error::{Result, IgtlError};
use serde_json::Value;
use std::fs;
use tracing::info;

/// Load an OpenIGTLink message from a JSON file (supports all 41 message types)
pub fn load_message_from_file(file_path: &str) -> Result<AnyMessage> {
    info!("Loading message from file: {}", file_path);
    let content = fs::read_to_string(file_path)?;
    load_message_from_json(&content)
}

/// Load an OpenIGTLink message from JSON string (supports all 41 message types)
pub fn load_message_from_json(json_str: &str) -> Result<AnyMessage> {
    let value: Value = serde_json::from_str(json_str).map_err(|e| {
        IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("JSON parse error: {}", e),
        ))
    })?;

    let device_name = value["device_name"]
        .as_str()
        .unwrap_or("DefaultDevice");

    let message_type = value["message_type"]
        .as_str()
        .ok_or_else(|| IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Missing 'message_type' field in JSON",
        )))?;

    let content = &value["content"];

    match message_type {
        // ===== CORE MESSAGE TYPES (6) =====
        "TRANSFORM" => {
            let matrix_arr = &value["transform"]["matrix"];
            let mut matrix = [[0.0f32; 4]; 4];

            for i in 0..4 {
                for j in 0..4 {
                    matrix[i][j] = matrix_arr[i][j]
                        .as_f64()
                        .unwrap_or(0.0) as f32;
                }
            }

            let transform = TransformMessage { matrix };
            let msg = IgtlMessage::new(transform, device_name)?;
            info!("✓ Loaded TRANSFORM message");
            Ok(AnyMessage::Transform(msg))
        }

        "STATUS" => {
            let code = content["code"].as_u64().unwrap_or(0) as u16;
            let status_string = content["message"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string();

            let status = StatusMessage {
                code,
                subcode: 0,
                error_name: String::new(),
                status_string,
            };
            let msg = IgtlMessage::new(status, device_name)?;
            info!("✓ Loaded STATUS message");
            Ok(AnyMessage::Status(msg))
        }

        "CAPABILITY" => {
            let types: Vec<String> = content["types"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            let capability = CapabilityMessage::new(types);
            let msg = IgtlMessage::new(capability, device_name)?;
            info!("✓ Loaded CAPABILITY message");
            Ok(AnyMessage::Capability(msg))
        }

        "STRING" => {
            let encoding = content["encoding"].as_u64().unwrap_or(3) as u16;
            let string_data = content["data"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let string_msg = StringMessage {
                encoding,
                string: string_data,
            };
            let msg = IgtlMessage::new(string_msg, device_name)?;
            info!("✓ Loaded STRING message");
            Ok(AnyMessage::String(msg))
        }

        "POSITION" => {
            let x = content["position"]["x"].as_f64().unwrap_or(0.0) as f32;
            let y = content["position"]["y"].as_f64().unwrap_or(0.0) as f32;
            let z = content["position"]["z"].as_f64().unwrap_or(0.0) as f32;

            let qx = content["quaternion"]["x"].as_f64().unwrap_or(0.0) as f32;
            let qy = content["quaternion"]["y"].as_f64().unwrap_or(0.0) as f32;
            let qz = content["quaternion"]["z"].as_f64().unwrap_or(0.0) as f32;
            let qw = content["quaternion"]["w"].as_f64().unwrap_or(1.0) as f32;

            let position = PositionMessage {
                position: [x, y, z],
                quaternion: [qx, qy, qz, qw],
            };
            let msg = IgtlMessage::new(position, device_name)?;
            info!("✓ Loaded POSITION message");
            Ok(AnyMessage::Position(msg))
        }

        "SENSOR" => {
            let status = content["status"].as_u64().unwrap_or(0) as u8;
            let unit = content["unit"].as_u64().unwrap_or(0);
            let data: Vec<f64> = content["data"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|v| v.as_f64())
                .collect();

            let sensor = SensorMessage {
                status,
                unit,
                data,
            };
            let msg = IgtlMessage::new(sensor, device_name)?;
            info!("✓ Loaded SENSOR message");
            Ok(AnyMessage::Sensor(msg))
        }

        // ===== QUERY MESSAGES (8) - Empty Body =====
        "GET_TRANSFORM" => {
            let msg = IgtlMessage::new(GetTransformMessage, device_name)?;
            info!("✓ Loaded GET_TRANSFORM message");
            Ok(AnyMessage::GetTransform(msg))
        }

        "GET_STATUS" => {
            let msg = IgtlMessage::new(GetStatusMessage, device_name)?;
            info!("✓ Loaded GET_STATUS message");
            Ok(AnyMessage::GetStatus(msg))
        }

        "GET_CAPABILITY" => {
            let msg = IgtlMessage::new(GetCapabilityMessage, device_name)?;
            info!("✓ Loaded GET_CAPABILITY message");
            Ok(AnyMessage::GetCapability(msg))
        }

        "GET_IMAGE" => {
            let msg = IgtlMessage::new(GetImageMessage, device_name)?;
            info!("✓ Loaded GET_IMAGE message");
            Ok(AnyMessage::GetImage(msg))
        }

        "GET_IMGMETA" => {
            let msg = IgtlMessage::new(GetImgMetaMessage, device_name)?;
            info!("✓ Loaded GET_IMGMETA message");
            Ok(AnyMessage::GetImgMeta(msg))
        }

        "GET_LBMETA" => {
            let msg = IgtlMessage::new(GetLbMetaMessage, device_name)?;
            info!("✓ Loaded GET_LBMETA message");
            Ok(AnyMessage::GetLbMeta(msg))
        }

        "GET_POINT" => {
            let msg = IgtlMessage::new(GetPointMessage, device_name)?;
            info!("✓ Loaded GET_POINT message");
            Ok(AnyMessage::GetPoint(msg))
        }

        "GET_TDATA" => {
            let msg = IgtlMessage::new(GetTDataMessage, device_name)?;
            info!("✓ Loaded GET_TDATA message");
            Ok(AnyMessage::GetTData(msg))
        }

        // ===== RESPONSE MESSAGES (5) =====
        "RTS_TRANSFORM" => {
            let code = content["code"].as_u64().unwrap_or(0) as u16;
            let rts = StatusMessage {
                code,
                subcode: 0,
                error_name: String::new(),
                status_string: String::new(),
            };
            let msg = IgtlMessage::new(rts, device_name)?;
            info!("✓ Loaded RTS_TRANSFORM message");
            Ok(AnyMessage::RtsTransform(msg))
        }

        "RTS_STATUS" => {
            let code = content["code"].as_u64().unwrap_or(0) as u16;
            let rts = StatusMessage {
                code,
                subcode: 0,
                error_name: String::new(),
                status_string: String::new(),
            };
            let msg = IgtlMessage::new(rts, device_name)?;
            info!("✓ Loaded RTS_STATUS message");
            Ok(AnyMessage::RtsStatus(msg))
        }

        "RTS_CAPABILITY" => {
            let code = content["code"].as_u64().unwrap_or(0) as u16;
            let rts = StatusMessage {
                code,
                subcode: 0,
                error_name: String::new(),
                status_string: String::new(),
            };
            let msg = IgtlMessage::new(rts, device_name)?;
            info!("✓ Loaded RTS_CAPABILITY message");
            Ok(AnyMessage::RtsCapability(msg))
        }

        "RTS_IMAGE" => {
            let code = content["code"].as_u64().unwrap_or(0) as u16;
            let rts = StatusMessage {
                code,
                subcode: 0,
                error_name: String::new(),
                status_string: String::new(),
            };
            let msg = IgtlMessage::new(rts, device_name)?;
            info!("✓ Loaded RTS_IMAGE message");
            Ok(AnyMessage::RtsImage(msg))
        }

        "RTS_TDATA" => {
            let status = content["status"].as_u64().unwrap_or(1) as u16;
            let rts = RtsTDataMessage { status };
            let msg = IgtlMessage::new(rts, device_name)?;
            info!("✓ Loaded RTS_TDATA message");
            Ok(AnyMessage::RtsTData(msg))
        }

        // ===== STREAMING CONTROL MESSAGES (7) =====
        "STT_TDATA" => {
            let resolution = value["resolution"].as_u64().unwrap_or(0) as u32;
            let coordinate_name = value["coordinate_name"].as_str().unwrap_or("").to_string();
            let start = StartTDataMessage {
                resolution,
                coordinate_name
            };
            let msg = IgtlMessage::new(start, device_name)?;
            info!("✓ Loaded STT_TDATA message");
            Ok(AnyMessage::StartTData(msg))
        }

        "STP_TRANSFORM" => {
            let msg = IgtlMessage::new(StopTransformMessage, device_name)?;
            info!("✓ Loaded STP_TRANSFORM message");
            Ok(AnyMessage::StopTransform(msg))
        }

        "STP_POSITION" => {
            let msg = IgtlMessage::new(StopPositionMessage, device_name)?;
            info!("✓ Loaded STP_POSITION message");
            Ok(AnyMessage::StopPosition(msg))
        }

        "STP_QTDATA" => {
            let msg = IgtlMessage::new(StopQtDataMessage, device_name)?;
            info!("✓ Loaded STP_QTDATA message");
            Ok(AnyMessage::StopQtData(msg))
        }

        "STP_TDATA" => {
            let msg = IgtlMessage::new(StopTDataMessage, device_name)?;
            info!("✓ Loaded STP_TDATA message");
            Ok(AnyMessage::StopTData(msg))
        }

        "STP_IMAGE" => {
            let msg = IgtlMessage::new(StopImageMessage, device_name)?;
            info!("✓ Loaded STP_IMAGE message");
            Ok(AnyMessage::StopImage(msg))
        }

        "STP_NDARRAY" => {
            let msg = IgtlMessage::new(StopNdArrayMessage, device_name)?;
            info!("✓ Loaded STP_NDARRAY message");
            Ok(AnyMessage::StopNdArray(msg))
        }

        // ===== ELEMENT ARRAY MESSAGES (8) =====
        "QTDATA" => {
            let mut elements = Vec::new();
            if let Some(arr) = content["elements"].as_array() {
                for elem in arr {
                    let name = elem["name"].as_str().unwrap_or("").to_string();
                    let instrument_type_val = elem["instrument_type"].as_u64().unwrap_or(0) as u8;
                    let instrument_type = unsafe { std::mem::transmute::<u8, InstrumentType>(instrument_type_val) };
                    let position = [
                        elem["position"][0].as_f64().unwrap_or(0.0) as f32,
                        elem["position"][1].as_f64().unwrap_or(0.0) as f32,
                        elem["position"][2].as_f64().unwrap_or(0.0) as f32,
                    ];
                    let quaternion = [
                        elem["quaternion"][0].as_f64().unwrap_or(0.0) as f32,
                        elem["quaternion"][1].as_f64().unwrap_or(0.0) as f32,
                        elem["quaternion"][2].as_f64().unwrap_or(0.0) as f32,
                        elem["quaternion"][3].as_f64().unwrap_or(1.0) as f32,
                    ];
                    elements.push(TrackingElement {
                        name,
                        instrument_type,
                        position,
                        quaternion,
                    });
                }
            }
            let qtdata = QtDataMessage { elements };
            let msg = IgtlMessage::new(qtdata, device_name)?;
            info!("✓ Loaded QTDATA message");
            Ok(AnyMessage::QtData(msg))
        }

        "TDATA" => {
            let mut elements = Vec::new();
            if let Some(arr) = content["elements"].as_array() {
                for elem in arr {
                    let name = elem["name"].as_str().unwrap_or("").to_string();
                    let instrument_type_val = elem["instrument_type"].as_u64().unwrap_or(0) as u8;
                    let instrument_type = unsafe { std::mem::transmute::<u8, TrackingInstrumentType>(instrument_type_val) };
                    let mut matrix = [[0.0f32; 4]; 3];
                    for i in 0..3 {
                        for j in 0..4 {
                            matrix[i][j] = elem["matrix"][i][j].as_f64().unwrap_or(0.0) as f32;
                        }
                    }
                    elements.push(TrackingDataElement {
                        name,
                        instrument_type,
                        matrix,
                    });
                }
            }
            let tdata = TDataMessage { elements };
            let msg = IgtlMessage::new(tdata, device_name)?;
            info!("✓ Loaded TDATA message");
            Ok(AnyMessage::TData(msg))
        }

        "POINT" => {
            let mut points = Vec::new();
            if let Some(arr) = content["points"].as_array() {
                for pt in arr {
                    let name = pt["name"].as_str().unwrap_or("").to_string();
                    let group = pt["group"].as_str().unwrap_or("").to_string();
                    let rgba = [
                        pt["rgba"][0].as_u64().unwrap_or(255) as u8,
                        pt["rgba"][1].as_u64().unwrap_or(255) as u8,
                        pt["rgba"][2].as_u64().unwrap_or(255) as u8,
                        pt["rgba"][3].as_u64().unwrap_or(255) as u8,
                    ];
                    let position = [
                        pt["position"][0].as_f64().unwrap_or(0.0) as f32,
                        pt["position"][1].as_f64().unwrap_or(0.0) as f32,
                        pt["position"][2].as_f64().unwrap_or(0.0) as f32,
                    ];
                    let diameter = pt["diameter"].as_f64().unwrap_or(1.0) as f32;
                    let owner = pt["owner"].as_str().unwrap_or("").to_string();
                    points.push(PointElement {
                        name,
                        group,
                        rgba,
                        position,
                        diameter,
                        owner,
                    });
                }
            }
            let point = PointMessage { points };
            let msg = IgtlMessage::new(point, device_name)?;
            info!("✓ Loaded POINT message");
            Ok(AnyMessage::Point(msg))
        }

        "TRAJECTORY" => {
            let mut trajectories = Vec::new();
            if let Some(arr) = content["trajectories"].as_array() {
                for traj in arr {
                    let name = traj["name"].as_str().unwrap_or("").to_string();
                    let group_name = traj["group_name"].as_str().unwrap_or("").to_string();
                    let trajectory_type_val = traj["trajectory_type"].as_u64().unwrap_or(3) as u8;
                    let trajectory_type = unsafe { std::mem::transmute::<u8, TrajectoryType>(trajectory_type_val) };
                    let rgba = [
                        traj["rgba"][0].as_u64().unwrap_or(255) as u8,
                        traj["rgba"][1].as_u64().unwrap_or(255) as u8,
                        traj["rgba"][2].as_u64().unwrap_or(255) as u8,
                        traj["rgba"][3].as_u64().unwrap_or(255) as u8,
                    ];
                    let entry_point = [
                        traj["entry_point"][0].as_f64().unwrap_or(0.0) as f32,
                        traj["entry_point"][1].as_f64().unwrap_or(0.0) as f32,
                        traj["entry_point"][2].as_f64().unwrap_or(0.0) as f32,
                    ];
                    let target_point = [
                        traj["target_point"][0].as_f64().unwrap_or(0.0) as f32,
                        traj["target_point"][1].as_f64().unwrap_or(0.0) as f32,
                        traj["target_point"][2].as_f64().unwrap_or(0.0) as f32,
                    ];
                    let diameter = traj["diameter"].as_f64().unwrap_or(1.0) as f32;
                    let owner_image = traj["owner_image"].as_str().unwrap_or("").to_string();
                    trajectories.push(TrajectoryElement {
                        name,
                        group_name,
                        trajectory_type,
                        rgba,
                        entry_point,
                        target_point,
                        diameter,
                        owner_image,
                    });
                }
            }
            let trajectory = TrajectoryMessage { trajectories };
            let msg = IgtlMessage::new(trajectory, device_name)?;
            info!("✓ Loaded TRAJECTORY message");
            Ok(AnyMessage::Trajectory(msg))
        }

        "BIND" => {
            let mut entries = Vec::new();
            if let Some(arr) = content["entries"].as_array() {
                for entry in arr {
                    let message_type = entry["message_type"].as_str().unwrap_or("").to_string();
                    let device_name_entry = entry["device_name"].as_str().unwrap_or("").to_string();
                    entries.push(BindEntry {
                        message_type,
                        device_name: device_name_entry,
                    });
                }
            }
            let bind = BindMessage { entries };
            let msg = IgtlMessage::new(bind, device_name)?;
            info!("✓ Loaded BIND message");
            Ok(AnyMessage::Bind(msg))
        }

        "COLORTABLE" => {
            let index_type_val = content["index_type"].as_u64().unwrap_or(3) as u16;
            let index_type = match index_type_val {
                3 => IndexType::Uint8,
                5 => IndexType::Uint16,
                _ => IndexType::Uint8, // Default to Uint8 for unknown values
            };
            let mut colors = Vec::new();
            if let Some(arr) = content["colors"].as_array() {
                for color in arr {
                    let rgba = [
                        color["rgba"][0].as_u64().unwrap_or(0) as u8,
                        color["rgba"][1].as_u64().unwrap_or(0) as u8,
                        color["rgba"][2].as_u64().unwrap_or(0) as u8,
                        color["rgba"][3].as_u64().unwrap_or(255) as u8,
                    ];
                    colors.push(ColorEntry { rgba });
                }
            }
            let colortable = ColorTableMessage {
                index_type,
                colors,
            };
            let msg = IgtlMessage::new(colortable, device_name)?;
            info!("✓ Loaded COLORTABLE message");
            Ok(AnyMessage::ColorTable(msg))
        }

        "IMGMETA" => {
            let mut images = Vec::new();
            if let Some(arr) = content["images"].as_array() {
                for img in arr {
                    let name = img["name"].as_str().unwrap_or("").to_string();
                    let id = img["id"].as_str().unwrap_or("").to_string();
                    let modality = img["modality"].as_str().unwrap_or("").to_string();
                    let patient_name = img["patient_name"].as_str().unwrap_or("").to_string();
                    let patient_id = img["patient_id"].as_str().unwrap_or("").to_string();
                    let timestamp = img["timestamp"].as_u64().unwrap_or(0);
                    let size = [
                        img["size"][0].as_u64().unwrap_or(0) as u16,
                        img["size"][1].as_u64().unwrap_or(0) as u16,
                        img["size"][2].as_u64().unwrap_or(0) as u16,
                    ];
                    let scalar_type = img["scalar_type"].as_u64().unwrap_or(9) as u8;
                    images.push(ImageMetaElement {
                        name,
                        id,
                        modality,
                        patient_name,
                        patient_id,
                        timestamp,
                        size,
                        scalar_type,
                    });
                }
            }
            let imgmeta = ImgMetaMessage { images };
            let msg = IgtlMessage::new(imgmeta, device_name)?;
            info!("✓ Loaded IMGMETA message");
            Ok(AnyMessage::ImgMeta(msg))
        }

        "LBMETA" => {
            let mut labels = Vec::new();
            if let Some(arr) = content["labels"].as_array() {
                for lbl in arr {
                    let name = lbl["name"].as_str().unwrap_or("").to_string();
                    let id = lbl["id"].as_str().unwrap_or("").to_string();
                    let label = lbl["label"].as_u64().unwrap_or(0) as u8;
                    let rgba = [
                        lbl["rgba"][0].as_u64().unwrap_or(0) as u8,
                        lbl["rgba"][1].as_u64().unwrap_or(0) as u8,
                        lbl["rgba"][2].as_u64().unwrap_or(0) as u8,
                        lbl["rgba"][3].as_u64().unwrap_or(255) as u8,
                    ];
                    let size = [
                        lbl["size"][0].as_u64().unwrap_or(0) as u16,
                        lbl["size"][1].as_u64().unwrap_or(0) as u16,
                        lbl["size"][2].as_u64().unwrap_or(0) as u16,
                    ];
                    let owner = lbl["owner"].as_str().unwrap_or("").to_string();
                    labels.push(LabelMetaElement {
                        name,
                        id,
                        label,
                        rgba,
                        size,
                        owner,
                    });
                }
            }
            let lbmeta = LbMetaMessage { labels };
            let msg = IgtlMessage::new(lbmeta, device_name)?;
            info!("✓ Loaded LBMETA message");
            Ok(AnyMessage::LbMeta(msg))
        }

        // ===== SIMPLE MESSAGE TYPES (4) =====
        "VIDEOMETA" => {
            let codec_val = content["codec"].as_u64().unwrap_or(5) as u16;
            let codec = match codec_val {
                1 => CodecType::H264,
                2 => CodecType::VP9,
                _ => CodecType::H264, // Default to H264
            };
            let width = content["width"].as_u64().unwrap_or(640) as u16;
            let height = content["height"].as_u64().unwrap_or(480) as u16;
            let framerate = content["framerate"].as_u64().unwrap_or(30) as u8;
            let bitrate = content["bitrate"].as_u64().unwrap_or(0) as u32;

            let videometa = VideoMetaMessage {
                codec,
                width,
                height,
                framerate,
                bitrate,
            };
            let msg = IgtlMessage::new(videometa, device_name)?;
            info!("✓ Loaded VIDEOMETA message");
            Ok(AnyMessage::VideoMeta(msg))
        }

        "COMMAND" => {
            let command_id = content["command_id"].as_u64().unwrap_or(0) as u32;
            let command_name = content["command_name"].as_str().unwrap_or("").to_string();
            let encoding = content["encoding"].as_u64().unwrap_or(3) as u16;
            let command_text = content["command"].as_str().unwrap_or("").to_string();

            let command = CommandMessage {
                command_id,
                command_name,
                encoding,
                command: command_text,
            };
            let msg = IgtlMessage::new(command, device_name)?;
            info!("✓ Loaded COMMAND message");
            Ok(AnyMessage::Command(msg))
        }

        "VIDEO" => {
            let codec_val = content["codec"].as_u64().unwrap_or(5) as u16;
            let codec = match codec_val {
                1 => CodecType::H264,
                2 => CodecType::VP9,
                _ => CodecType::H264, // Default to H264
            };
            let width = content["width"].as_u64().unwrap_or(640) as u16;
            let height = content["height"].as_u64().unwrap_or(480) as u16;
            let frame_data = Vec::new();  // Empty frame data for now

            let video = VideoMessage {
                codec,
                width,
                height,
                frame_data,
            };
            let msg = IgtlMessage::new(video, device_name)?;
            info!("✓ Loaded VIDEO message");
            Ok(AnyMessage::Video(msg))
        }

        "POLYDATA" => {
            let mut points_vec = Vec::new();
            if let Some(arr) = content["points"].as_array() {
                for pt in arr {
                    if let Some(pt_arr) = pt.as_array() {
                        if pt_arr.len() >= 3 {
                            points_vec.push([
                                pt_arr[0].as_f64().unwrap_or(0.0) as f32,
                                pt_arr[1].as_f64().unwrap_or(0.0) as f32,
                                pt_arr[2].as_f64().unwrap_or(0.0) as f32,
                            ]);
                        }
                    }
                }
            }

            let vertices: Vec<u32> = content["vertices"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|v| v.as_u64().map(|u| u as u32))
                .collect();

            let lines: Vec<u32> = content["lines"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|v| v.as_u64().map(|u| u as u32))
                .collect();

            let polygons: Vec<u32> = content["polygons"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|v| v.as_u64().map(|u| u as u32))
                .collect();

            let triangle_strips: Vec<u32> = content["triangle_strips"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|v| v.as_u64().map(|u| u as u32))
                .collect();

            let polydata = PolyDataMessage {
                points: points_vec,
                vertices,
                lines,
                polygons,
                triangle_strips,
                attributes: Vec::new(),
            };
            let msg = IgtlMessage::new(polydata, device_name)?;
            info!("✓ Loaded POLYDATA message");
            Ok(AnyMessage::PolyData(msg))
        }

        "NDARRAY" => {
            let scalar_type_val = content["scalar_type"].as_u64().unwrap_or(9) as u16;
            let scalar_type = match scalar_type_val {
                2 => ScalarType::Int8,
                3 => ScalarType::Uint8,
                4 => ScalarType::Int16,
                5 => ScalarType::Uint16,
                6 => ScalarType::Int32,
                7 => ScalarType::Uint32,
                10 => ScalarType::Float32,
                11 => ScalarType::Float64,
                _ => ScalarType::Uint8, // Default to Uint8
            };
            let size: Vec<u16> = content["size"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|v| v.as_u64().map(|u| u as u16))
                .collect();
            let data = Vec::new();  // Empty binary data for now

            let ndarray = NdArrayMessage {
                scalar_type,
                size,
                data,
            };
            let msg = IgtlMessage::new(ndarray, device_name)?;
            info!("✓ Loaded NDARRAY message");
            Ok(AnyMessage::NdArray(msg))
        }

        "IMAGE" => {
            let scalar_type_val = content["scalar_type"].as_u64().unwrap_or(3) as u8;
            let scalar_type = match scalar_type_val {
                2 => ImageScalarType::Int8,
                3 => ImageScalarType::Uint8,
                4 => ImageScalarType::Int16,
                5 => ImageScalarType::Uint16,
                6 => ImageScalarType::Int32,
                7 => ImageScalarType::Uint32,
                10 => ImageScalarType::Float32,
                11 => ImageScalarType::Float64,
                _ => ImageScalarType::Uint8,
            };
            let size = [
                content["size"][0].as_u64().unwrap_or(256) as u16,
                content["size"][1].as_u64().unwrap_or(256) as u16,
                content["size"][2].as_u64().unwrap_or(1) as u16,
            ];
            let data = vec![0u8; (size[0] as usize * size[1] as usize * size[2] as usize * 3 * scalar_type.size())];

            let image = ImageMessage::rgb(scalar_type, size, data)?;
            let msg = IgtlMessage::new(image, device_name)?;
            info!("✓ Loaded IMAGE message");
            Ok(AnyMessage::Image(msg))
        }

        _ => {
            Err(IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unknown message type: {}", message_type),
            )))
        }
    }
}

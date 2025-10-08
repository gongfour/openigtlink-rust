//! Message compression support for OpenIGTLink
//!
//! Provides compression/decompression functionality for large messages
//! (images, video, point clouds) to reduce network bandwidth.
//!
//! # Supported Algorithms
//!
//! - **Deflate (zlib)**: Standard compression, good balance of speed and ratio
//! - **Gzip**: Compatible with standard gzip format
//! - **None**: No compression (passthrough)
//!
//! # Examples
//!
//! ```
//! use openigtlink_rust::compression::{compress, decompress, CompressionLevel, CompressionType};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = vec![0u8; 10000]; // Large message
//!
//! // Compress with default level
//! let compressed = compress(&data, CompressionType::Deflate, CompressionLevel::Default)?;
//! println!("Original: {} bytes, Compressed: {} bytes", data.len(), compressed.len());
//!
//! // Decompress
//! let decompressed = decompress(&compressed, CompressionType::Deflate)?;
//! assert_eq!(data, decompressed);
//! # Ok(())
//! # }
//! ```

use crate::error::{IgtlError, Result};
use flate2::read::{DeflateDecoder, GzDecoder};
use flate2::write::{DeflateEncoder, GzEncoder};
use flate2::Compression;
use std::io::{Read, Write};
use tracing::{debug, info, trace};

/// Compression algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// No compression
    None,
    /// Deflate (zlib) compression
    Deflate,
    /// Gzip compression
    Gzip,
}

impl CompressionType {
    /// Get the compression type name
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Deflate => "deflate",
            Self::Gzip => "gzip",
        }
    }

    /// Check if compression is enabled
    pub fn is_compressed(&self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Compression level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// No compression (level 0)
    None,
    /// Fast compression, lower ratio (level 1)
    Fast,
    /// Default compression (level 6)
    Default,
    /// Best compression, slower (level 9)
    Best,
    /// Custom level (0-9)
    Custom(u32),
}

impl CompressionLevel {
    /// Convert to flate2 Compression level
    fn to_flate2(self) -> Compression {
        match self {
            Self::None => Compression::none(),
            Self::Fast => Compression::fast(),
            Self::Default => Compression::default(),
            Self::Best => Compression::best(),
            Self::Custom(level) => Compression::new(level),
        }
    }

    /// Get numeric level value
    pub fn level(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::Fast => 1,
            Self::Default => 6,
            Self::Best => 9,
            Self::Custom(level) => *level,
        }
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::Default
    }
}

/// Compress data using the specified algorithm and level
///
/// # Arguments
///
/// * `data` - Raw data to compress
/// * `compression_type` - Compression algorithm to use
/// * `level` - Compression level
///
/// # Returns
///
/// Compressed data
///
/// # Examples
///
/// ```
/// use openigtlink_rust::compression::{compress, CompressionLevel, CompressionType};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let data = vec![0u8; 1000];
/// let compressed = compress(&data, CompressionType::Deflate, CompressionLevel::Best)?;
/// assert!(compressed.len() < data.len());
/// # Ok(())
/// # }
/// ```
pub fn compress(
    data: &[u8],
    compression_type: CompressionType,
    level: CompressionLevel,
) -> Result<Vec<u8>> {
    trace!(
        compression_type = compression_type.name(),
        level = level.level(),
        input_size = data.len(),
        "Starting compression"
    );

    let compressed = match compression_type {
        CompressionType::None => {
            debug!("No compression requested, returning original data");
            data.to_vec()
        }
        CompressionType::Deflate => {
            let mut encoder = DeflateEncoder::new(Vec::new(), level.to_flate2());
            encoder.write_all(data).map_err(|e| {
                IgtlError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Deflate compression failed: {}", e),
                ))
            })?;
            encoder.finish().map_err(|e| {
                IgtlError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Deflate compression finish failed: {}", e),
                ))
            })?
        }
        CompressionType::Gzip => {
            let mut encoder = GzEncoder::new(Vec::new(), level.to_flate2());
            encoder.write_all(data).map_err(|e| {
                IgtlError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Gzip compression failed: {}", e),
                ))
            })?;
            encoder.finish().map_err(|e| {
                IgtlError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Gzip compression finish failed: {}", e),
                ))
            })?
        }
    };

    let ratio = if !data.is_empty() {
        (compressed.len() as f64 / data.len() as f64) * 100.0
    } else {
        0.0
    };

    info!(
        compression_type = compression_type.name(),
        level = level.level(),
        original_size = data.len(),
        compressed_size = compressed.len(),
        ratio_pct = format!("{:.1}%", ratio),
        "Compression completed"
    );

    Ok(compressed)
}

/// Decompress data using the specified algorithm
///
/// # Arguments
///
/// * `data` - Compressed data
/// * `compression_type` - Compression algorithm used
///
/// # Returns
///
/// Decompressed data
///
/// # Examples
///
/// ```
/// use openigtlink_rust::compression::{compress, decompress, CompressionLevel, CompressionType};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let original = vec![1, 2, 3, 4, 5];
/// let compressed = compress(&original, CompressionType::Deflate, CompressionLevel::Default)?;
/// let decompressed = decompress(&compressed, CompressionType::Deflate)?;
/// assert_eq!(original, decompressed);
/// # Ok(())
/// # }
/// ```
pub fn decompress(data: &[u8], compression_type: CompressionType) -> Result<Vec<u8>> {
    trace!(
        compression_type = compression_type.name(),
        compressed_size = data.len(),
        "Starting decompression"
    );

    let decompressed = match compression_type {
        CompressionType::None => {
            debug!("No decompression needed, returning original data");
            data.to_vec()
        }
        CompressionType::Deflate => {
            let mut decoder = DeflateDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).map_err(|e| {
                IgtlError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Deflate decompression failed: {}", e),
                ))
            })?;
            decompressed
        }
        CompressionType::Gzip => {
            let mut decoder = GzDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).map_err(|e| {
                IgtlError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Gzip decompression failed: {}", e),
                ))
            })?;
            decompressed
        }
    };

    info!(
        compression_type = compression_type.name(),
        compressed_size = data.len(),
        decompressed_size = decompressed.len(),
        "Decompression completed"
    );

    Ok(decompressed)
}

/// Compression statistics
#[derive(Debug, Clone)]
pub struct CompressionStats {
    /// Original data size
    pub original_size: usize,
    /// Compressed data size
    pub compressed_size: usize,
    /// Compression ratio (compressed/original)
    pub ratio: f64,
    /// Space saved in bytes
    pub space_saved: usize,
    /// Compression type used
    pub compression_type: CompressionType,
    /// Compression level used
    pub level: CompressionLevel,
}

impl CompressionStats {
    /// Calculate statistics for a compression operation
    pub fn calculate(
        original_size: usize,
        compressed_size: usize,
        compression_type: CompressionType,
        level: CompressionLevel,
    ) -> Self {
        let ratio = if original_size > 0 {
            compressed_size as f64 / original_size as f64
        } else {
            0.0
        };

        let space_saved = original_size.saturating_sub(compressed_size);

        Self {
            original_size,
            compressed_size,
            ratio,
            space_saved,
            compression_type,
            level,
        }
    }

    /// Get compression ratio as percentage
    pub fn ratio_percent(&self) -> f64 {
        self.ratio * 100.0
    }

    /// Get space saved as percentage
    pub fn space_saved_percent(&self) -> f64 {
        if self.original_size > 0 {
            (self.space_saved as f64 / self.original_size as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_compression() {
        let data = vec![1, 2, 3, 4, 5];
        let compressed = compress(&data, CompressionType::None, CompressionLevel::Default).unwrap();
        assert_eq!(data, compressed);

        let decompressed = decompress(&compressed, CompressionType::None).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_deflate_compression() {
        let data = vec![0u8; 1000]; // Highly compressible
        let compressed =
            compress(&data, CompressionType::Deflate, CompressionLevel::Default).unwrap();

        // Should be much smaller
        assert!(compressed.len() < data.len());
        assert!(compressed.len() < 100); // Should compress very well

        let decompressed = decompress(&compressed, CompressionType::Deflate).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_gzip_compression() {
        let data = vec![1u8; 1000];
        let compressed = compress(&data, CompressionType::Gzip, CompressionLevel::Default).unwrap();

        assert!(compressed.len() < data.len());

        let decompressed = decompress(&compressed, CompressionType::Gzip).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_compression_levels() {
        let data = vec![0u8; 10000];

        let fast = compress(&data, CompressionType::Deflate, CompressionLevel::Fast).unwrap();
        let default = compress(&data, CompressionType::Deflate, CompressionLevel::Default).unwrap();
        let best = compress(&data, CompressionType::Deflate, CompressionLevel::Best).unwrap();

        // Best should be smallest (or equal for highly compressible data)
        assert!(best.len() <= default.len());
        assert!(default.len() <= fast.len() || default.len() < 100); // May be same for zeros

        // All should decompress correctly
        assert_eq!(data, decompress(&fast, CompressionType::Deflate).unwrap());
        assert_eq!(
            data,
            decompress(&default, CompressionType::Deflate).unwrap()
        );
        assert_eq!(data, decompress(&best, CompressionType::Deflate).unwrap());
    }

    #[test]
    fn test_random_data_compression() {
        // Random data is not very compressible
        let data: Vec<u8> = (0..1000).map(|i| (i * 37 % 256) as u8).collect();

        let compressed =
            compress(&data, CompressionType::Deflate, CompressionLevel::Default).unwrap();

        // May not compress much, but should still work
        let decompressed = decompress(&compressed, CompressionType::Deflate).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_empty_data() {
        let data = vec![];
        let compressed =
            compress(&data, CompressionType::Deflate, CompressionLevel::Default).unwrap();
        let decompressed = decompress(&compressed, CompressionType::Deflate).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_compression_stats() {
        let stats = CompressionStats::calculate(
            1000,
            500,
            CompressionType::Deflate,
            CompressionLevel::Default,
        );

        assert_eq!(stats.original_size, 1000);
        assert_eq!(stats.compressed_size, 500);
        assert_eq!(stats.ratio, 0.5);
        assert_eq!(stats.space_saved, 500);
        assert_eq!(stats.ratio_percent(), 50.0);
        assert_eq!(stats.space_saved_percent(), 50.0);
    }

    #[test]
    fn test_compression_type_names() {
        assert_eq!(CompressionType::None.name(), "none");
        assert_eq!(CompressionType::Deflate.name(), "deflate");
        assert_eq!(CompressionType::Gzip.name(), "gzip");
    }

    #[test]
    fn test_compression_level_values() {
        assert_eq!(CompressionLevel::None.level(), 0);
        assert_eq!(CompressionLevel::Fast.level(), 1);
        assert_eq!(CompressionLevel::Default.level(), 6);
        assert_eq!(CompressionLevel::Best.level(), 9);
        assert_eq!(CompressionLevel::Custom(5).level(), 5);
    }
}

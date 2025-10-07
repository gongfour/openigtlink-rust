//! Partial message transfer for large messages
//!
//! Supports chunked transfer of large messages (images, video) with
//! resume capability and progress tracking.

use crate::error::{IgtlError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, trace, warn};

/// Configuration for partial message transfer
#[derive(Debug, Clone)]
pub struct TransferConfig {
    /// Size of each chunk in bytes
    pub chunk_size: usize,
    /// Whether to allow resume after interruption
    pub allow_resume: bool,
    /// Timeout for transfer session (None = no timeout)
    pub timeout_secs: Option<u64>,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            chunk_size: 65536,      // 64KB chunks
            allow_resume: true,
            timeout_secs: Some(300), // 5 minutes
        }
    }
}

/// Unique identifier for a transfer session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferId(u64);

impl TransferId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

/// State of a partial transfer session
#[derive(Debug, Clone)]
pub enum TransferState {
    /// Transfer is in progress
    InProgress {
        /// Bytes transferred so far
        bytes_transferred: usize,
        /// Total bytes to transfer
        total_bytes: usize,
        /// Current chunk index
        chunk_index: usize,
    },
    /// Transfer completed successfully
    Completed {
        /// Total bytes transferred
        total_bytes: usize,
    },
    /// Transfer was interrupted
    Interrupted {
        /// Bytes transferred before interruption
        bytes_transferred: usize,
        /// Total bytes
        total_bytes: usize,
        /// Can resume from this point
        resumable: bool,
    },
    /// Transfer failed
    Failed {
        /// Error message
        error: String,
    },
}

impl TransferState {
    /// Get progress percentage (0.0 - 1.0)
    pub fn progress(&self) -> f64 {
        match self {
            Self::InProgress { bytes_transferred, total_bytes, .. } => {
                if *total_bytes > 0 {
                    (*bytes_transferred as f64) / (*total_bytes as f64)
                } else {
                    0.0
                }
            }
            Self::Completed { .. } => 1.0,
            Self::Interrupted { bytes_transferred, total_bytes, .. } => {
                if *total_bytes > 0 {
                    (*bytes_transferred as f64) / (*total_bytes as f64)
                } else {
                    0.0
                }
            }
            Self::Failed { .. } => 0.0,
        }
    }

    /// Check if transfer is complete
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    /// Check if transfer can be resumed
    pub fn is_resumable(&self) -> bool {
        matches!(self, Self::Interrupted { resumable: true, .. })
    }
}

/// Information about a transfer session
#[derive(Debug, Clone)]
pub struct TransferInfo {
    pub id: TransferId,
    pub state: TransferState,
    pub config: TransferConfig,
    pub started_at: std::time::Instant,
    pub updated_at: std::time::Instant,
}

impl TransferInfo {
    /// Get elapsed time since transfer started
    pub fn elapsed(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }

    /// Get time since last update
    pub fn idle_time(&self) -> std::time::Duration {
        self.updated_at.elapsed()
    }

    /// Calculate transfer speed in bytes/sec
    pub fn speed_bps(&self) -> f64 {
        match &self.state {
            TransferState::InProgress { bytes_transferred, .. }
            | TransferState::Interrupted { bytes_transferred, .. } => {
                let elapsed_secs = self.elapsed().as_secs_f64();
                if elapsed_secs > 0.0 {
                    (*bytes_transferred as f64) / elapsed_secs
                } else {
                    0.0
                }
            }
            TransferState::Completed { total_bytes } => {
                let elapsed_secs = self.elapsed().as_secs_f64();
                if elapsed_secs > 0.0 {
                    (*total_bytes as f64) / elapsed_secs
                } else {
                    0.0
                }
            }
            TransferState::Failed { .. } => 0.0,
        }
    }
}

/// Manager for partial message transfers
pub struct PartialTransferManager {
    transfers: Arc<Mutex<HashMap<TransferId, TransferInfo>>>,
    next_id: Arc<Mutex<u64>>,
    config: TransferConfig,
}

impl PartialTransferManager {
    /// Create a new transfer manager with default configuration
    pub fn new() -> Self {
        Self::with_config(TransferConfig::default())
    }

    /// Create a new transfer manager with custom configuration
    pub fn with_config(config: TransferConfig) -> Self {
        info!(
            chunk_size = config.chunk_size,
            allow_resume = config.allow_resume,
            timeout_secs = ?config.timeout_secs,
            "Creating partial transfer manager"
        );
        Self {
            transfers: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            config,
        }
    }

    /// Start a new transfer session
    ///
    /// # Arguments
    /// * `total_bytes` - Total size of data to transfer
    ///
    /// # Returns
    /// Transfer ID for the new session
    pub async fn start_transfer(&self, total_bytes: usize) -> Result<TransferId> {
        let mut next_id = self.next_id.lock().await;
        let id = TransferId(*next_id);
        *next_id += 1;
        drop(next_id);

        info!(
            transfer_id = id.value(),
            total_bytes = total_bytes,
            chunk_size = self.config.chunk_size,
            "Starting new transfer"
        );

        let now = std::time::Instant::now();
        let info = TransferInfo {
            id,
            state: TransferState::InProgress {
                bytes_transferred: 0,
                total_bytes,
                chunk_index: 0,
            },
            config: self.config.clone(),
            started_at: now,
            updated_at: now,
        };

        self.transfers.lock().await.insert(id, info);
        Ok(id)
    }

    /// Update transfer progress
    ///
    /// # Arguments
    /// * `id` - Transfer ID
    /// * `bytes_transferred` - Total bytes transferred so far
    /// * `chunk_index` - Current chunk index
    pub async fn update_progress(
        &self,
        id: TransferId,
        bytes_transferred: usize,
        chunk_index: usize,
    ) -> Result<()> {
        let mut transfers = self.transfers.lock().await;

        let info = transfers.get_mut(&id).ok_or_else(|| {
            warn!(transfer_id = id.value(), "Transfer not found");
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Transfer not found",
            ))
        })?;

        if let TransferState::InProgress { total_bytes, .. } = info.state {
            let progress = (bytes_transferred as f64 / total_bytes as f64) * 100.0;
            trace!(
                transfer_id = id.value(),
                bytes_transferred = bytes_transferred,
                total_bytes = total_bytes,
                chunk_index = chunk_index,
                progress_pct = format!("{:.1}%", progress),
                "Transfer progress updated"
            );
            info.state = TransferState::InProgress {
                bytes_transferred,
                total_bytes,
                chunk_index,
            };
            info.updated_at = std::time::Instant::now();
        } else {
            warn!(transfer_id = id.value(), "Transfer is not in progress");
            return Err(IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Transfer is not in progress",
            )));
        }

        Ok(())
    }

    /// Mark transfer as completed
    pub async fn complete_transfer(&self, id: TransferId) -> Result<()> {
        let mut transfers = self.transfers.lock().await;

        let info = transfers.get_mut(&id).ok_or_else(|| {
            warn!(transfer_id = id.value(), "Transfer not found");
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Transfer not found",
            ))
        })?;

        if let TransferState::InProgress { total_bytes, .. } = info.state {
            let elapsed = info.elapsed().as_secs_f64();
            let speed_mbps = if elapsed > 0.0 {
                (total_bytes as f64) / elapsed / 1_000_000.0
            } else {
                0.0
            };
            info!(
                transfer_id = id.value(),
                total_bytes = total_bytes,
                elapsed_secs = format!("{:.2}", elapsed),
                speed_mbps = format!("{:.2}", speed_mbps),
                "Transfer completed"
            );
            info.state = TransferState::Completed { total_bytes };
            info.updated_at = std::time::Instant::now();
        }

        Ok(())
    }

    /// Interrupt transfer (can be resumed if configured)
    pub async fn interrupt_transfer(&self, id: TransferId) -> Result<()> {
        let mut transfers = self.transfers.lock().await;

        let info = transfers.get_mut(&id).ok_or_else(|| {
            warn!(transfer_id = id.value(), "Transfer not found");
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Transfer not found",
            ))
        })?;

        if let TransferState::InProgress {
            bytes_transferred,
            total_bytes,
            ..
        } = info.state
        {
            let resumable = info.config.allow_resume;
            warn!(
                transfer_id = id.value(),
                bytes_transferred = bytes_transferred,
                total_bytes = total_bytes,
                resumable = resumable,
                "Transfer interrupted"
            );
            info.state = TransferState::Interrupted {
                bytes_transferred,
                total_bytes,
                resumable,
            };
            info.updated_at = std::time::Instant::now();
        }

        Ok(())
    }

    /// Resume an interrupted transfer
    pub async fn resume_transfer(&self, id: TransferId) -> Result<usize> {
        let mut transfers = self.transfers.lock().await;

        let info = transfers.get_mut(&id).ok_or_else(|| {
            warn!(transfer_id = id.value(), "Transfer not found");
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Transfer not found",
            ))
        })?;

        match info.state {
            TransferState::Interrupted {
                bytes_transferred,
                total_bytes,
                resumable: true,
            } => {
                let chunk_index = bytes_transferred / info.config.chunk_size;
                info!(
                    transfer_id = id.value(),
                    resuming_from = bytes_transferred,
                    total_bytes = total_bytes,
                    chunk_index = chunk_index,
                    "Resuming transfer"
                );
                info.state = TransferState::InProgress {
                    bytes_transferred,
                    total_bytes,
                    chunk_index,
                };
                info.updated_at = std::time::Instant::now();
                Ok(bytes_transferred)
            }
            TransferState::Interrupted { resumable: false, .. } => {
                warn!(transfer_id = id.value(), "Transfer is not resumable");
                Err(IgtlError::Io(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Transfer is not resumable",
                    ),
                ))
            }
            _ => {
                warn!(transfer_id = id.value(), "Transfer is not interrupted");
                Err(IgtlError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Transfer is not interrupted",
                )))
            }
        }
    }

    /// Fail a transfer with error message
    pub async fn fail_transfer(&self, id: TransferId, error: String) -> Result<()> {
        let mut transfers = self.transfers.lock().await;

        if let Some(info) = transfers.get_mut(&id) {
            warn!(
                transfer_id = id.value(),
                error = %error,
                "Transfer failed"
            );
            info.state = TransferState::Failed { error };
            info.updated_at = std::time::Instant::now();
        }

        Ok(())
    }

    /// Get transfer information
    pub async fn get_transfer(&self, id: TransferId) -> Option<TransferInfo> {
        self.transfers.lock().await.get(&id).cloned()
    }

    /// Get all active transfers
    pub async fn active_transfers(&self) -> Vec<TransferInfo> {
        self.transfers
            .lock()
            .await
            .values()
            .filter(|info| matches!(info.state, TransferState::InProgress { .. }))
            .cloned()
            .collect()
    }

    /// Remove completed or failed transfers
    pub async fn cleanup_completed(&self) {
        self.transfers.lock().await.retain(|_, info| {
            !matches!(
                info.state,
                TransferState::Completed { .. } | TransferState::Failed { .. }
            )
        });
    }

    /// Remove transfers that have timed out
    pub async fn cleanup_timed_out(&self) {
        let config = &self.config;
        if let Some(timeout_secs) = config.timeout_secs {
            let timeout = std::time::Duration::from_secs(timeout_secs);
            let mut transfers = self.transfers.lock().await;
            let before_count = transfers.len();
            transfers.retain(|id, info| {
                let keep = info.idle_time() < timeout;
                if !keep {
                    debug!(
                        transfer_id = id.value(),
                        idle_time_secs = info.idle_time().as_secs(),
                        "Removing timed out transfer"
                    );
                }
                keep
            });
            let removed = before_count - transfers.len();
            if removed > 0 {
                info!(removed_count = removed, "Cleaned up timed out transfers");
            }
        }
    }
}

impl Default for PartialTransferManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_transfer() {
        let manager = PartialTransferManager::new();
        let id = manager.start_transfer(1000000).await.unwrap();

        let info = manager.get_transfer(id).await.unwrap();
        assert!(matches!(info.state, TransferState::InProgress { .. }));
    }

    #[tokio::test]
    async fn test_update_progress() {
        let manager = PartialTransferManager::new();
        let id = manager.start_transfer(1000000).await.unwrap();

        manager.update_progress(id, 500000, 5).await.unwrap();

        let info = manager.get_transfer(id).await.unwrap();
        assert_eq!(info.state.progress(), 0.5);
    }

    #[tokio::test]
    async fn test_complete_transfer() {
        let manager = PartialTransferManager::new();
        let id = manager.start_transfer(1000000).await.unwrap();

        manager.update_progress(id, 1000000, 10).await.unwrap();
        manager.complete_transfer(id).await.unwrap();

        let info = manager.get_transfer(id).await.unwrap();
        assert!(info.state.is_complete());
        assert_eq!(info.state.progress(), 1.0);
    }

    #[tokio::test]
    async fn test_interrupt_and_resume() {
        let manager = PartialTransferManager::new();
        let id = manager.start_transfer(1000000).await.unwrap();

        manager.update_progress(id, 500000, 5).await.unwrap();
        manager.interrupt_transfer(id).await.unwrap();

        let info = manager.get_transfer(id).await.unwrap();
        assert!(info.state.is_resumable());

        let resumed_at = manager.resume_transfer(id).await.unwrap();
        assert_eq!(resumed_at, 500000);

        let info = manager.get_transfer(id).await.unwrap();
        assert!(matches!(info.state, TransferState::InProgress { .. }));
    }

    #[tokio::test]
    async fn test_fail_transfer() {
        let manager = PartialTransferManager::new();
        let id = manager.start_transfer(1000000).await.unwrap();

        manager
            .fail_transfer(id, "Network error".to_string())
            .await
            .unwrap();

        let info = manager.get_transfer(id).await.unwrap();
        assert!(matches!(info.state, TransferState::Failed { .. }));
    }

    #[tokio::test]
    async fn test_active_transfers() {
        let manager = PartialTransferManager::new();

        let id1 = manager.start_transfer(1000000).await.unwrap();
        let id2 = manager.start_transfer(2000000).await.unwrap();
        let id3 = manager.start_transfer(3000000).await.unwrap();

        manager.complete_transfer(id1).await.unwrap();
        manager.interrupt_transfer(id2).await.unwrap();

        let active = manager.active_transfers().await;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, id3);
    }

    #[tokio::test]
    async fn test_cleanup_completed() {
        let manager = PartialTransferManager::new();

        let id1 = manager.start_transfer(1000000).await.unwrap();
        let id2 = manager.start_transfer(2000000).await.unwrap();

        manager.complete_transfer(id1).await.unwrap();

        manager.cleanup_completed().await;

        assert!(manager.get_transfer(id1).await.is_none());
        assert!(manager.get_transfer(id2).await.is_some());
    }

    #[tokio::test]
    async fn test_transfer_speed() {
        let manager = PartialTransferManager::new();
        let id = manager.start_transfer(1000000).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        manager.update_progress(id, 500000, 5).await.unwrap();

        let info = manager.get_transfer(id).await.unwrap();
        let speed = info.speed_bps();

        // Should be around 5 MB/s (500000 bytes in ~0.1 sec)
        assert!(speed > 1_000_000.0); // At least 1 MB/s
    }
}

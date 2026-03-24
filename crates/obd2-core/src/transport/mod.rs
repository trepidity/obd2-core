//! Transport trait and built-in implementations.
//!
//! A transport represents the physical connection to an OBD-II adapter
//! (serial port, BLE GATT, WiFi socket, etc.). This is an open trait —
//! anyone can implement it for custom transports.

pub mod mock;
pub mod logging;
#[cfg(feature = "serial")]
pub mod serial;
#[cfg(feature = "ble")]
pub mod ble;
#[cfg(feature = "ble")]
pub use ble::{ADAPTER_NAME_PATTERNS, is_adapter_match};
pub use logging::{LoggingTransport, CaptureMetadata, parse_raw_capture};

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use crate::error::Obd2Error;

/// Callback invoked on each raw read chunk before response assembly.
/// Used by LoggingTransport to record transport-level reads.
pub type ChunkObserver = Arc<Mutex<dyn Fn(&[u8]) + Send>>;

/// Physical connection to an OBD-II adapter.
///
/// Implementors handle the raw byte-level communication over a specific
/// medium (serial, BLE, WiFi, etc.). The adapter layer builds on top
/// of this to implement protocol-specific commands.
///
/// # Example
///
/// ```rust,no_run
/// use obd2_core::transport::Transport;
/// use obd2_core::error::Obd2Error;
/// use async_trait::async_trait;
///
/// struct MyTransport;
///
/// #[async_trait]
/// impl Transport for MyTransport {
///     async fn write(&mut self, data: &[u8]) -> Result<(), Obd2Error> { Ok(()) }
///     async fn read(&mut self) -> Result<Vec<u8>, Obd2Error> { Ok(vec![]) }
///     async fn reset(&mut self) -> Result<(), Obd2Error> { Ok(()) }
///     fn name(&self) -> &str { "my-transport" }
///     fn set_chunk_observer(&mut self, _observer: Option<obd2_core::transport::ChunkObserver>) {}
/// }
/// ```
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send raw bytes to the adapter.
    async fn write(&mut self, data: &[u8]) -> Result<(), Obd2Error>;

    /// Read a response from the adapter.
    /// Returns the complete response as bytes (including any framing).
    async fn read(&mut self) -> Result<Vec<u8>, Obd2Error>;

    /// Reset the physical connection.
    async fn reset(&mut self) -> Result<(), Obd2Error>;

    /// Human-readable transport name for logging.
    fn name(&self) -> &str;

    /// Set a callback invoked on each raw read chunk during `read()`.
    /// Default implementation does nothing.
    fn set_chunk_observer(&mut self, _observer: Option<ChunkObserver>) {}
}

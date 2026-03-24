//! Logging transport decorator for raw protocol capture.
//!
//! Wraps any `Transport` and records all write/read operations to a
//! human-readable `.obd2raw` text file.

use std::io::{self, Write as IoWrite, BufWriter};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;

use super::{Transport, ChunkObserver};
use crate::error::Obd2Error;

/// Escape bytes for the log file.
/// Printable ASCII (0x20-0x7E) rendered literally, except backslash is escaped.
/// Special cases: \r, \n, \t. Everything else: \xHH.
fn escape_bytes(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len());
    for &b in data {
        match b {
            b'\r' => out.push_str("\\r"),
            b'\n' => out.push_str("\\n"),
            b'\t' => out.push_str("\\t"),
            b'\\' => out.push_str("\\\\"),
            0x20..=0x7E => out.push(b as char),
            _ => out.push_str(&format!("\\x{:02X}", b)),
        }
    }
    out
}

/// Metadata written to the .obd2raw file header.
pub struct CaptureMetadata {
    pub transport_type: String,
    pub port_or_device: String,
    pub baud_rate: Option<u32>,
}

/// Format the file header comment lines.
fn format_header(meta: &CaptureMetadata) -> String {
    let mut header = String::from("# obd2-raw v1\n");
    if meta.baud_rate.is_some() {
        header.push_str(&format!(
            "# transport={} port={} baud={}\n",
            meta.transport_type,
            meta.port_or_device,
            meta.baud_rate.unwrap(),
        ));
    } else {
        header.push_str(&format!(
            "# transport={} device={}\n",
            meta.transport_type,
            meta.port_or_device,
        ));
    }
    header.push_str(&format!(
        "# started={}\n",
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    ));
    header
}

/// Timestamped byte chunks collected from the chunk observer callback.
type ChunkBuf = Arc<Mutex<Vec<(f64, Vec<u8>)>>>;

/// A transport decorator that records all wire-level communication.
///
/// Wraps any `Transport` and logs every `write()` and `read()` to a
/// `.obd2raw` text file when capture is active. Zero overhead when inactive.
pub struct LoggingTransport<T: Transport> {
    inner: T,
    writer: Option<BufWriter<File>>,
    start_instant: Instant,
    chunk_buf: ChunkBuf,
}

impl<T: Transport> LoggingTransport<T> {
    /// Wrap a transport. Capture starts inactive.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            writer: None,
            start_instant: Instant::now(),
            chunk_buf: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start capturing to a file. Writes the header.
    pub fn start_capture(
        &mut self,
        path: &Path,
        metadata: &CaptureMetadata,
    ) -> io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(format_header(metadata).as_bytes())?;
        self.writer = Some(writer);
        self.start_instant = Instant::now();
        self.install_chunk_observer();
        Ok(())
    }

    /// Stop capturing. Flushes and closes the file.
    pub fn stop_capture(&mut self) -> io::Result<Option<PathBuf>> {
        self.inner.set_chunk_observer(None);
        if let Some(mut w) = self.writer.take() {
            w.flush()?;
        }
        Ok(None)
    }

    /// Whether capture is currently active.
    pub fn is_capturing(&self) -> bool {
        self.writer.is_some()
    }

    /// Access the inner transport.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Access the inner transport mutably.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Elapsed seconds since capture start.
    fn elapsed_secs(&self) -> f64 {
        self.start_instant.elapsed().as_secs_f64()
    }

    /// Write a log line if capture is active.
    fn log_line(&mut self, direction: char, data: &[u8]) {
        let ts = self.elapsed_secs();
        if let Some(ref mut w) = self.writer {
            let _ = writeln!(w, "{:.3} {} {}", ts, direction, escape_bytes(data));
        }
    }

    /// Flush any buffered chunk observations as R.chunk lines.
    fn flush_chunks(&mut self) {
        if let Some(ref mut w) = self.writer {
            if let Ok(mut chunks) = self.chunk_buf.lock() {
                for (ts, data) in chunks.drain(..) {
                    let _ = writeln!(w, "{:.3} R.chunk {}", ts, escape_bytes(&data));
                }
            }
        }
    }

    /// Install a chunk observer on the inner transport.
    fn install_chunk_observer(&mut self) {
        let buf = self.chunk_buf.clone();
        let start = self.start_instant;
        let observer: ChunkObserver = Arc::new(Mutex::new(move |data: &[u8]| {
            let ts = start.elapsed().as_secs_f64();
            if let Ok(mut chunks) = buf.lock() {
                chunks.push((ts, data.to_vec()));
            }
        }));
        self.inner.set_chunk_observer(Some(observer));
    }
}

#[async_trait]
impl<T: Transport> Transport for LoggingTransport<T> {
    async fn write(&mut self, data: &[u8]) -> Result<(), Obd2Error> {
        self.log_line('W', data);
        self.inner.write(data).await
    }

    async fn read(&mut self) -> Result<Vec<u8>, Obd2Error> {
        let result = self.inner.read().await?;
        self.flush_chunks();
        self.log_line('R', &result);
        Ok(result)
    }

    async fn reset(&mut self) -> Result<(), Obd2Error> {
        self.inner.reset().await
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn set_chunk_observer(&mut self, observer: Option<ChunkObserver>) {
        self.inner.set_chunk_observer(observer);
    }
}

/// Reverse the escape_bytes encoding back to raw bytes.
fn unescape_str(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('r') => out.push(b'\r'),
                Some('n') => out.push(b'\n'),
                Some('t') => out.push(b'\t'),
                Some('\\') => out.push(b'\\'),
                Some('x') => {
                    let hi = chars.next().unwrap_or('0');
                    let lo = chars.next().unwrap_or('0');
                    let hex: String = [hi, lo].iter().collect();
                    out.push(u8::from_str_radix(&hex, 16).unwrap_or(0));
                }
                Some(other) => {
                    out.push(b'\\');
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(other.encode_utf8(&mut buf).as_bytes());
                }
                None => out.push(b'\\'),
            }
        } else {
            let mut buf = [0u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
    }
    out
}

/// Parse a .obd2raw file into (command, response) pairs.
///
/// Filters to `W` and `R` lines (ignoring `R.chunk`), pairs them
/// sequentially, and unescapes the byte encoding. Commands have
/// trailing `\r` stripped for direct use with `MockTransport::expect()`.
pub fn parse_raw_capture(path: &Path) -> io::Result<Vec<(String, String)>> {
    let content = std::fs::read_to_string(path)?;
    let mut pairs = Vec::new();
    let mut pending_write: Option<String> = None;

    for line in content.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        // Format: "0.000 W ATZ\r" or "0.328 R 41 0C 0A A0\r\r>"
        let mut parts = line.splitn(3, ' ');
        let _timestamp = parts.next();
        let direction = match parts.next() {
            Some(d) => d,
            None => continue,
        };
        let payload = parts.next().unwrap_or("");

        match direction {
            "W" => {
                let raw = unescape_str(payload);
                // Strip trailing \r (ELM327 command framing)
                let cmd = if raw.last() == Some(&b'\r') {
                    String::from_utf8_lossy(&raw[..raw.len() - 1]).to_string()
                } else {
                    String::from_utf8_lossy(&raw).to_string()
                };
                pending_write = Some(cmd);
            }
            "R" => {
                if let Some(cmd) = pending_write.take() {
                    let raw = unescape_str(payload);
                    let response = String::from_utf8_lossy(&raw).to_string();
                    pairs.push((cmd, response));
                }
            }
            _ => {} // R.chunk and anything else — skip
        }
    }

    Ok(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockTransport;
    use tempfile::NamedTempFile;

    // ── escape_bytes tests ──────────────────────────────────────────────

    #[test]
    fn test_escape_printable_ascii() {
        assert_eq!(escape_bytes(b"ATZ"), "ATZ");
        assert_eq!(escape_bytes(b"010C"), "010C");
    }

    #[test]
    fn test_escape_cr_and_prompt() {
        assert_eq!(escape_bytes(b"41 0C 0A A0\r\r>"), "41 0C 0A A0\\r\\r>");
    }

    #[test]
    fn test_escape_newline_tab() {
        assert_eq!(escape_bytes(b"OK\r\n"), "OK\\r\\n");
        assert_eq!(escape_bytes(b"\t"), "\\t");
    }

    #[test]
    fn test_escape_backslash() {
        assert_eq!(escape_bytes(b"a\\b"), "a\\\\b");
    }

    #[test]
    fn test_escape_non_printable() {
        assert_eq!(escape_bytes(&[0x00, 0x01, 0xFF]), "\\x00\\x01\\xFF");
    }

    #[test]
    fn test_escape_mixed_command() {
        assert_eq!(escape_bytes(b"ATZ\r"), "ATZ\\r");
    }

    #[test]
    fn test_escape_elm327_full_response() {
        assert_eq!(
            escape_bytes(b"010C\r41 0C 0A A0\r\r>"),
            "010C\\r41 0C 0A A0\\r\\r>"
        );
    }

    // ── CaptureMetadata tests ───────────────────────────────────────────

    #[test]
    fn test_capture_metadata_header() {
        let meta = CaptureMetadata {
            transport_type: "serial".to_string(),
            port_or_device: "/dev/tty.usbserial-0001".to_string(),
            baud_rate: Some(115200),
        };
        let header = format_header(&meta);
        assert!(header.starts_with("# obd2-raw v1\n"));
        assert!(header.contains("transport=serial"));
        assert!(header.contains("port=/dev/tty.usbserial-0001"));
        assert!(header.contains("baud=115200"));
        assert!(header.contains("# started="));
    }

    #[test]
    fn test_capture_metadata_header_ble() {
        let meta = CaptureMetadata {
            transport_type: "ble".to_string(),
            port_or_device: "OBDLink MX+".to_string(),
            baud_rate: None,
        };
        let header = format_header(&meta);
        assert!(header.contains("transport=ble"));
        assert!(header.contains("device=OBDLink MX+"));
        assert!(!header.contains("baud="));
    }

    // ── LoggingTransport integration tests ──────────────────────────────

    #[tokio::test]
    async fn test_logging_transport_captures_write_read() {
        let mut mock = MockTransport::new();
        mock.expect("ATZ", "ELM327 v2.1\r>");
        mock.expect("010C", "41 0C 0A A0\r>");

        let mut lt = LoggingTransport::new(mock);
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        lt.start_capture(
            &path,
            &CaptureMetadata {
                transport_type: "mock".to_string(),
                port_or_device: "test".to_string(),
                baud_rate: None,
            },
        ).unwrap();

        // Send ATZ
        lt.write(b"ATZ\r").await.unwrap();
        let resp = lt.read().await.unwrap();
        assert!(String::from_utf8_lossy(&resp).contains("ELM327"));

        // Send 010C
        lt.write(b"010C\r").await.unwrap();
        let resp = lt.read().await.unwrap();
        assert!(String::from_utf8_lossy(&resp).contains("41 0C"));

        lt.stop_capture().unwrap();

        // Read the log file and verify content
        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Header
        assert!(lines[0].starts_with("# obd2-raw v1"));
        assert!(lines[1].contains("transport=mock"));
        assert!(lines[2].starts_with("# started="));

        // Data lines (skip header)
        let data_lines: Vec<&str> = lines.iter().filter(|l| !l.starts_with('#')).copied().collect();
        assert_eq!(data_lines.len(), 4); // W, R, W, R

        assert!(data_lines[0].contains(" W ATZ\\r"));
        assert!(data_lines[1].contains(" R ELM327 v2.1\\r>"));
        assert!(data_lines[2].contains(" W 010C\\r"));
        assert!(data_lines[3].contains(" R 41 0C 0A A0\\r>"));
    }

    #[tokio::test]
    async fn test_logging_transport_inactive_passthrough() {
        let mut mock = MockTransport::new();
        mock.expect("ATZ", "OK\r>");

        let mut lt = LoggingTransport::new(mock);
        // Do NOT start capture
        lt.write(b"ATZ\r").await.unwrap();
        let resp = lt.read().await.unwrap();
        assert!(String::from_utf8_lossy(&resp).contains("OK"));
        // No crash, no file created — just passthrough
        assert!(!lt.is_capturing());
    }

    #[tokio::test]
    async fn test_logging_transport_forwarding() {
        let mut mock = MockTransport::new();
        mock.expect("ATZ", "OK\r>");

        let mut lt = LoggingTransport::new(mock);
        lt.write(b"ATZ\r").await.unwrap();
        let resp = lt.read().await.unwrap();
        assert_eq!(String::from_utf8_lossy(&resp), "OK\r>");

        // name() forwards
        assert_eq!(lt.name(), "mock");
    }

    // ── parse_raw_capture tests ─────────────────────────────────────────

    #[test]
    fn test_parse_raw_capture_basic() {
        let content = "\
# obd2-raw v1
# transport=serial port=/dev/ttyUSB0 baud=115200
# started=2026-03-24T14:30:00.000Z
0.000 W ATZ\\r
0.150 R ELM327 v2.1\\r\\r>
0.200 W 010C\\r
0.328 R 41 0C 0A A0\\r\\r>
";
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), content).unwrap();

        let pairs = parse_raw_capture(tmp.path()).unwrap();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0, "ATZ");
        assert_eq!(pairs[0].1, "ELM327 v2.1\r\r>");
        assert_eq!(pairs[1].0, "010C");
        assert_eq!(pairs[1].1, "41 0C 0A A0\r\r>");
    }

    #[test]
    fn test_parse_raw_capture_ignores_chunks() {
        let content = "\
# obd2-raw v1
# transport=serial port=/dev/ttyUSB0 baud=115200
# started=2026-03-24T14:30:00.000Z
0.000 W ATZ\\r
0.045 R.chunk ELM327 v2.
0.089 R.chunk 1\\r\\r>
0.089 R ELM327 v2.1\\r\\r>
";
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), content).unwrap();

        let pairs = parse_raw_capture(tmp.path()).unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "ATZ");
        assert_eq!(pairs[0].1, "ELM327 v2.1\r\r>");
    }

    #[test]
    fn test_parse_raw_capture_strips_trailing_cr() {
        let content = "\
# obd2-raw v1
# transport=mock device=test
# started=2026-03-24T14:30:00.000Z
0.000 W ATE0\\r
0.050 R ATE0\\rOK\\r\\r>
";
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), content).unwrap();

        let pairs = parse_raw_capture(tmp.path()).unwrap();
        assert_eq!(pairs[0].0, "ATE0");
        assert_eq!(pairs[0].1, "ATE0\rOK\r\r>");
    }
}

//! Diagnostic-session state and utility types owned by `Session`.

use tokio::sync::watch;

/// State of the active diagnostic session.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SessionState {
    /// Default session — no special access.
    #[default]
    Default,
    /// Extended session active — module unlock state is tracked per session.
    Extended {
        /// Modules that have been security-unlocked.
        unlocked_modules: Vec<String>,
    },
    /// Programming session (for reflash — the library does not provide flashing).
    Programming,
}

/// Callback type for computing a security key from a seed.
pub type KeyFunction = Box<dyn Fn(&[u8]) -> Vec<u8> + Send>;

/// Create a cancel channel for tester-present keepalive ownership inside `Session`.
#[allow(dead_code)]
pub fn start_tester_present_keepalive() -> watch::Sender<bool> {
    let (cancel_tx, _cancel_rx) = watch::channel(false);
    cancel_tx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_default() {
        assert_eq!(SessionState::default(), SessionState::Default);
    }

    #[test]
    fn test_keepalive_channel_defaults_to_not_cancelled() {
        let tx = start_tester_present_keepalive();
        assert!(!*tx.borrow());
    }
}

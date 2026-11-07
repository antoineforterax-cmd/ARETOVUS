use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeError {
    InvalidArguments,
    InvalidDataDir,
    BridgeBusy,
    BridgeUnavailable,
    BridgeTimeout,
    VerificationFailed,
    InvalidResponse,
    ExplorerUnavailable,
    ExplorerTimeout,
    ExplorerFailed,
    InvalidExplorerResponse,
}

impl NativeError {
    // Only fixed codes cross into the WebView. Paths and errors from child
    // processes cannot become fields in this payload.
    pub(crate) fn into_payload(self) -> Value {
        let code = match self {
            Self::InvalidArguments => "invalid_arguments",
            Self::InvalidDataDir => "invalid_data_dir",
            Self::BridgeBusy => "bridge_busy",
            Self::BridgeUnavailable => "bridge_unavailable",
            Self::BridgeTimeout => "bridge_timeout",
            Self::VerificationFailed => "verification_failed",
            Self::InvalidResponse => "invalid_response",
            Self::ExplorerUnavailable => "explorer_unavailable",
            Self::ExplorerTimeout => "explorer_timeout",
            Self::ExplorerFailed => "explorer_failed",
            Self::InvalidExplorerResponse => "invalid_explorer_response",
        };
        json!({ "schema_version": 1, "code": code })
    }
}

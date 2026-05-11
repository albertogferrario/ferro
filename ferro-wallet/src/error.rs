//! `WalletError` — the single error type for the ferro-wallet crate.
//!
//! Each variant's `Display` impl prefixes its name (`"config: …"`, `"apple sign: …"`)
//! so production log greps stay surgical. `Io(#[from] std::io::Error)` covers zip + io
//! plumbing in the apple package path.

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    /// Configuration error (missing env var or invalid value).
    #[error("config: {0}")]
    Config(String),

    /// Apple PKCS#7 signing failed (cert/key parse, sign call, DER serialization).
    #[error("apple sign: {0}")]
    AppleSign(String),

    /// Apple `.pkpass` packaging failed (manifest JSON build, zip assembly).
    #[error("apple package: {0}")]
    ApplePackage(String),

    /// Google save JWT signing failed (private key parse, RS256 encode).
    #[error("google jwt: {0}")]
    GoogleJwt(String),

    /// Image pipeline error (decode, resize, encode).
    #[error("image: {0}")]
    Image(String),

    /// QR code generation error.
    #[error("qr: {0}")]
    Qr(String),

    /// Caller supplied invalid input (out-of-range value, malformed string).
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// I/O error — zip writer, file handles, etc.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_config_displays_message() {
        let e = WalletError::Config("APP_NAME not set".into());
        assert_eq!(e.to_string(), "config: APP_NAME not set");
    }

    #[test]
    fn error_apple_sign_displays_message() {
        let e = WalletError::AppleSign("cert parse failed".into());
        assert_eq!(e.to_string(), "apple sign: cert parse failed");
    }

    #[test]
    fn error_apple_package_displays_message() {
        let e = WalletError::ApplePackage("zip write failed".into());
        assert_eq!(e.to_string(), "apple package: zip write failed");
    }

    #[test]
    fn error_google_jwt_displays_message() {
        let e = WalletError::GoogleJwt("private key parse failed".into());
        assert_eq!(e.to_string(), "google jwt: private key parse failed");
    }

    #[test]
    fn error_image_displays_message() {
        let e = WalletError::Image("decode failed".into());
        assert_eq!(e.to_string(), "image: decode failed");
    }

    #[test]
    fn error_qr_displays_message() {
        let e = WalletError::Qr("encode failed".into());
        assert_eq!(e.to_string(), "qr: encode failed");
    }

    #[test]
    fn error_invalid_input_displays_message() {
        let e = WalletError::InvalidInput("hex must be 6 chars".into());
        assert_eq!(e.to_string(), "invalid input: hex must be 6 chars");
    }

    #[test]
    fn error_io_displays_message() {
        let e = WalletError::Io(std::io::Error::other("disk full"));
        assert_eq!(e.to_string(), "io: disk full");
    }

    #[test]
    fn io_from_std_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let wallet_err: WalletError = WalletError::from(io_err);
        assert!(matches!(wallet_err, WalletError::Io(_)));
    }
}

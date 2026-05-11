//! Apple Wallet PKCS#7 detached signing.
//!
//! `SigningMaterial::parse` loads the signing cert, private key (optionally
//! passphrase-protected), and WWDR intermediate from PEM strings.
//! `sign_detached(manifest_bytes)` produces DER-encoded PKCS#7 with
//! `DETACHED | BINARY` flags. The WWDR cert is pushed onto a non-empty
//! `Stack<X509>` so the on-device chain is well-formed (RESEARCH.md Pitfall 2).

use openssl::pkcs7::{Pkcs7, Pkcs7Flags};
use openssl::pkey::{PKey, Private};
use openssl::stack::Stack;
use openssl::x509::X509;

use crate::WalletError;

pub(crate) struct SigningMaterial {
    pub cert: X509,
    pub key: PKey<Private>,
    pub wwdr: X509,
}

impl SigningMaterial {
    pub fn parse(
        cert_pem: &str,
        key_pem: &str,
        key_password: Option<&str>,
        wwdr_pem: &str,
    ) -> Result<Self, WalletError> {
        let cert = X509::from_pem(cert_pem.as_bytes())
            .map_err(|e| WalletError::AppleSign(format!("cert parse: {e}")))?;
        let key = match key_password {
            Some(pw) => PKey::private_key_from_pem_passphrase(key_pem.as_bytes(), pw.as_bytes()),
            None => PKey::private_key_from_pem(key_pem.as_bytes()),
        }
        .map_err(|e| WalletError::AppleSign(format!("key parse: {e}")))?;
        let wwdr = X509::from_pem(wwdr_pem.as_bytes())
            .map_err(|e| WalletError::AppleSign(format!("wwdr parse: {e}")))?;
        Ok(Self { cert, key, wwdr })
    }

    pub fn sign_detached(&self, manifest_bytes: &[u8]) -> Result<Vec<u8>, WalletError> {
        let mut wwdr_stack: Stack<X509> =
            Stack::new().map_err(|e| WalletError::AppleSign(format!("stack init: {e}")))?;
        wwdr_stack
            .push(self.wwdr.clone())
            .map_err(|e| WalletError::AppleSign(format!("stack push wwdr: {e}")))?;

        let flags = Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY;
        let pkcs7 = Pkcs7::sign(&self.cert, &self.key, &wwdr_stack, manifest_bytes, flags)
            .map_err(|e| WalletError::AppleSign(format!("pkcs7 sign: {e}")))?;
        pkcs7
            .to_der()
            .map_err(|e| WalletError::AppleSign(format!("pkcs7 to_der: {e}")))
    }
}

use crate::CryptoError;
use ed25519_dalek::{
    self as dalek, SigningKey, VerifyingKey,
    Signer as DalekSigner, Verifier as DalekVerifier,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// An Ed25519 signing keypair.
#[derive(ZeroizeOnDrop)]
pub struct Keypair(SigningKey);

impl Keypair {
    /// Generate a fresh random keypair.
    pub fn generate() -> Self {
        Keypair(SigningKey::generate(&mut OsRng))
    }

    /// Restore a keypair from 32 raw secret-key bytes.
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Keypair(SigningKey::from_bytes(bytes))
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.verifying_key())
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        Signature(self.0.sign(msg))
    }

    pub fn secret_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

/// An Ed25519 public (verifying) key — 32 bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey(#[serde(with = "verifying_key_serde")] VerifyingKey);

impl PublicKey {
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, CryptoError> {
        VerifyingKey::from_bytes(bytes)
            .map(PublicKey)
            .map_err(|_| CryptoError::InvalidPublicKey)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    pub fn verify(&self, msg: &[u8], sig: &Signature) -> Result<(), CryptoError> {
        self.0
            .verify(msg, &sig.0)
            .map_err(|_| CryptoError::VerificationFailed)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0.as_bytes())
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({})", &self.to_hex()[..12])
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// An Ed25519 signature — 64 bytes.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(#[serde(with = "signature_serde")] dalek::Signature);

impl Signature {
    pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self, CryptoError> {
        Ok(Signature(dalek::Signature::from_bytes(bytes)))
    }

    pub fn as_bytes(&self) -> [u8; 64] {
        self.0.to_bytes()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0.to_bytes())
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sig({}…)", &self.to_hex()[..12])
    }
}

// ---------- serde helpers ----------

mod verifying_key_serde {
    use ed25519_dalek::VerifyingKey;
    use serde::{Deserializer, Serializer, Deserialize};

    pub fn serialize<S: Serializer>(k: &VerifyingKey, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(k.as_bytes())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<VerifyingKey, D::Error> {
        let bytes: Vec<u8> = Vec::deserialize(d)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| serde::de::Error::custom("bad length"))?;
        VerifyingKey::from_bytes(&arr).map_err(serde::de::Error::custom)
    }
}

mod signature_serde {
    use ed25519_dalek::Signature as DalekSig;
    use serde::{Deserializer, Serializer, Deserialize};

    pub fn serialize<S: Serializer>(sig: &DalekSig, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(&sig.to_bytes())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<DalekSig, D::Error> {
        let bytes: Vec<u8> = Vec::deserialize(d)?;
        let arr: [u8; 64] = bytes.try_into().map_err(|_| serde::de::Error::custom("bad length"))?;
        Ok(DalekSig::from_bytes(&arr))

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify() {
        let kp = Keypair::generate();
        let msg = b"test message";
        let sig = kp.sign(msg);
        assert!(kp.public_key().verify(msg, &sig).is_ok());
    }

    #[test]
    fn wrong_message_fails() {
        let kp = Keypair::generate();
        let sig = kp.sign(b"hello");
        assert!(kp.public_key().verify(b"world", &sig).is_err());
    }

    #[test]
    fn keypair_roundtrip() {
        let kp = Keypair::generate();
        let bytes = kp.secret_bytes();
        let kp2 = Keypair::from_bytes(&bytes);
        assert_eq!(kp.public_key(), kp2.public_key());
    }
}

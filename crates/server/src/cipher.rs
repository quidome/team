//! The encryption of the player names (AES-256-GCM). A stored name is the version byte `1`, a new
//! random 12-byte nonce, then the encrypted name with its 16-byte tag. The name of the column is
//! the additional authenticated data, so a value moved to the other column does not decrypt.
//! Errors never contain a name or the key.

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

const VERSION: u8 = 1;
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

/// The key of the player names (`PLAYER_NAME_KEY`): 32 bytes. Its debug output never shows them.
#[derive(Clone)]
pub struct NameKey(Aes256Gcm);

impl std::fmt::Debug for NameKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NameKey(..)")
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum KeyError {
    #[error("must be standard base64")]
    NotBase64,
    #[error("must be {KEY_LEN} bytes, not {0}")]
    WrongLength(usize),
}

impl NameKey {
    /// Standard base64 of exactly 32 bytes; spaces and line ends around it are ignored.
    pub fn from_base64(text: &str) -> Result<Self, KeyError> {
        let bytes = STANDARD
            .decode(text.trim())
            .map_err(|_| KeyError::NotBase64)?;
        if bytes.len() != KEY_LEN {
            return Err(KeyError::WrongLength(bytes.len()));
        }
        Aes256Gcm::new_from_slice(&bytes)
            .map(Self)
            .map_err(|_| KeyError::WrongLength(bytes.len()))
    }
}

/// The column a stored name belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameField {
    First,
    Last,
}

impl NameField {
    fn aad(self) -> &'static [u8] {
        match self {
            Self::First => b"players.first_name",
            Self::Last => b"players.last_name",
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CipherError {
    #[error("unknown version {0}")]
    UnknownVersion(u8),
    #[error("too short")]
    TooShort,
    #[error("not authentic (wrong key or changed data)")]
    NotAuthentic,
    #[error("not text")]
    NotText,
    #[error("cannot encrypt")]
    Encrypt,
}

/// Encrypts a name for its column, with a new nonce each time.
pub fn seal(key: &NameKey, field: NameField, name: &str) -> Result<Vec<u8>, CipherError> {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted = key
        .0
        .encrypt(
            &nonce,
            Payload {
                msg: name.as_bytes(),
                aad: field.aad(),
            },
        )
        .map_err(|_| CipherError::Encrypt)?;
    let mut sealed = Vec::with_capacity(1 + NONCE_LEN + encrypted.len());
    sealed.push(VERSION);
    sealed.extend_from_slice(&nonce);
    sealed.extend_from_slice(&encrypted);
    Ok(sealed)
}

/// Decrypts a name that `seal` made for the same column with the same key.
pub fn open(key: &NameKey, field: NameField, sealed: &[u8]) -> Result<String, CipherError> {
    let (&version, rest) = sealed.split_first().ok_or(CipherError::TooShort)?;
    if version != VERSION {
        return Err(CipherError::UnknownVersion(version));
    }
    if rest.len() < NONCE_LEN + TAG_LEN {
        return Err(CipherError::TooShort);
    }
    let (nonce, encrypted) = rest.split_at(NONCE_LEN);
    let plain = key
        .0
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: encrypted,
                aad: field.aad(),
            },
        )
        .map_err(|_| CipherError::NotAuthentic)?;
    String::from_utf8(plain).map_err(|_| CipherError::NotText)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Result<NameKey, KeyError> {
        NameKey::from_base64(&STANDARD.encode([byte; KEY_LEN]))
    }

    #[test]
    fn a_sealed_name_opens_again() -> Result<(), Box<dyn std::error::Error>> {
        let key = key(7)?;
        let long = "x".repeat(100);
        for name in ["Anna", "Zoë", "Şükrü", long.as_str()] {
            for field in [NameField::First, NameField::Last] {
                let sealed = seal(&key, field, name)?;
                assert_eq!(open(&key, field, &sealed)?, name);
            }
        }
        Ok(())
    }

    #[test]
    fn the_same_name_is_sealed_differently_each_time() -> Result<(), Box<dyn std::error::Error>> {
        let key = key(7)?;
        let first = seal(&key, NameField::First, "Anna")?;
        let second = seal(&key, NameField::First, "Anna")?;
        assert_ne!(first, second);
        assert_eq!(first.len(), 1 + NONCE_LEN + "Anna".len() + TAG_LEN);
        Ok(())
    }

    #[test]
    fn another_key_does_not_open_it() -> Result<(), Box<dyn std::error::Error>> {
        let sealed = seal(&key(7)?, NameField::First, "Anna")?;
        assert_eq!(
            open(&key(8)?, NameField::First, &sealed),
            Err(CipherError::NotAuthentic)
        );
        Ok(())
    }

    #[test]
    fn a_first_name_does_not_open_as_a_last_name() -> Result<(), Box<dyn std::error::Error>> {
        let key = key(7)?;
        let sealed = seal(&key, NameField::First, "Anna")?;
        assert_eq!(
            open(&key, NameField::Last, &sealed),
            Err(CipherError::NotAuthentic)
        );
        Ok(())
    }

    #[test]
    fn changed_short_or_unknown_values_are_refused() -> Result<(), Box<dyn std::error::Error>> {
        let key = key(7)?;
        let sealed = seal(&key, NameField::First, "Anna")?;

        let mut changed = sealed.clone();
        if let Some(byte) = changed.get_mut(1 + NONCE_LEN) {
            *byte ^= 1;
        }
        assert_eq!(
            open(&key, NameField::First, &changed),
            Err(CipherError::NotAuthentic)
        );

        let mut other_version = sealed;
        other_version[0] = 2;
        assert_eq!(
            open(&key, NameField::First, &other_version),
            Err(CipherError::UnknownVersion(2))
        );

        assert_eq!(
            open(&key, NameField::First, &[1, 0, 0, 0, 0]),
            Err(CipherError::TooShort)
        );
        assert_eq!(
            open(&key, NameField::First, &[]),
            Err(CipherError::TooShort)
        );
        Ok(())
    }

    #[test]
    fn a_key_must_be_base64_of_32_bytes() {
        assert_eq!(
            NameKey::from_base64("not base64!").err(),
            Some(KeyError::NotBase64)
        );
        for length in [16, 31, 33] {
            assert_eq!(
                NameKey::from_base64(&STANDARD.encode(vec![7; length])).err(),
                Some(KeyError::WrongLength(length))
            );
        }
        assert!(NameKey::from_base64(&format!("{}\n", STANDARD.encode([7; KEY_LEN]))).is_ok());
    }

    #[test]
    fn debug_output_hides_the_key() -> Result<(), KeyError> {
        assert_eq!(format!("{:?}", key(7)?), "NameKey(..)");
        Ok(())
    }
}

//! Sérialisation canonique expérimentale (hypothèse TX-02, non validée).
//!
//! Règles de ce prototype :
//! - entiers à largeur fixe, **big-endian** (aucun encodage à longueur variable, donc aucune
//!   forme non canonique possible pour un entier) ;
//! - tableaux de taille fixe écrits tels quels ;
//! - listes et blocs d'octets : longueur sur 4 octets big-endian, **bornée** par un maximum
//!   connu du décodeur ;
//! - le décodage refuse les octets en trop, les longueurs excessives et les valeurs inconnues.
//!
//! Propriété visée et testée : si `decode(octets)` réussit, alors `encode(valeur) == octets`.

use std::fmt;

/// Erreur d'encodage : une valeur dépasse une limite connue du format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeError {
    /// Une longueur dépasse le maximum autorisé.
    LengthTooLarge {
        /// Longueur demandée.
        len: usize,
        /// Maximum autorisé.
        max: usize,
    },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::LengthTooLarge { len, max } => {
                write!(f, "longueur {len} au-dessus du maximum {max}")
            }
        }
    }
}

impl std::error::Error for EncodeError {}

/// Erreur de décodage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// Les données s'arrêtent avant la fin de la structure.
    UnexpectedEof,
    /// Des octets restent après la fin de la structure.
    TrailingBytes {
        /// Nombre d'octets restants.
        remaining: usize,
    },
    /// Une longueur annoncée dépasse le maximum autorisé.
    LengthTooLarge {
        /// Longueur annoncée.
        len: u64,
        /// Maximum autorisé.
        max: usize,
    },
    /// Une valeur est interdite par les règles du format.
    InvalidValue(&'static str),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::UnexpectedEof => write!(f, "fin de données inattendue"),
            DecodeError::TrailingBytes { remaining } => {
                write!(f, "{remaining} octet(s) en trop après la structure")
            }
            DecodeError::LengthTooLarge { len, max } => {
                write!(f, "longueur annoncée {len} au-dessus du maximum {max}")
            }
            DecodeError::InvalidValue(why) => write!(f, "valeur invalide : {why}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Écrivain d'octets pour l'encodage canonique.
#[derive(Debug, Default)]
pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    /// Crée un encodeur vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Écrit un octet.
    pub fn put_u8(&mut self, value: u8) {
        self.buf.push(value);
    }

    /// Écrit un entier 32 bits en big-endian.
    pub fn put_u32(&mut self, value: u32) {
        self.buf.extend_from_slice(&value.to_be_bytes());
    }

    /// Écrit un entier 64 bits en big-endian.
    pub fn put_u64(&mut self, value: u64) {
        self.buf.extend_from_slice(&value.to_be_bytes());
    }

    /// Écrit des octets de taille connue du format, sans préfixe de longueur.
    pub fn put_fixed(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// Écrit une longueur (4 octets big-endian) après avoir vérifié qu'elle ne dépasse pas `max`.
    pub fn put_len(&mut self, len: usize, max: usize) -> Result<(), EncodeError> {
        if len > max {
            return Err(EncodeError::LengthTooLarge { len, max });
        }
        let len32 = u32::try_from(len).map_err(|_| EncodeError::LengthTooLarge { len, max })?;
        self.put_u32(len32);
        Ok(())
    }

    /// Écrit un bloc d'octets précédé de sa longueur, bornée par `max`.
    pub fn put_var_bytes(&mut self, bytes: &[u8], max: usize) -> Result<(), EncodeError> {
        self.put_len(bytes.len(), max)?;
        self.put_fixed(bytes);
        Ok(())
    }

    /// Termine l'encodage et rend les octets.
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

/// Lecteur d'octets pour le décodage canonique.
#[derive(Debug)]
pub struct Decoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    /// Crée un décodeur sur `data`.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Nombre d'octets restant à lire.
    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if n > self.remaining() {
            return Err(DecodeError::UnexpectedEof);
        }
        let data: &'a [u8] = self.data;
        let slice = &data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    /// Lit un octet.
    pub fn get_u8(&mut self) -> Result<u8, DecodeError> {
        let b = self.take(1)?;
        Ok(b[0])
    }

    /// Lit un entier 32 bits big-endian.
    pub fn get_u32(&mut self) -> Result<u32, DecodeError> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Lit un entier 64 bits big-endian.
    pub fn get_u64(&mut self) -> Result<u64, DecodeError> {
        let b = self.take(8)?;
        Ok(u64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    /// Lit exactement `N` octets.
    pub fn get_array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let slice = self.take(N)?;
        let mut out = [0u8; N];
        out.copy_from_slice(slice);
        Ok(out)
    }

    /// Lit une longueur et refuse tout ce qui dépasse `max`.
    pub fn get_len(&mut self, max: usize) -> Result<usize, DecodeError> {
        let raw = self.get_u32()?;
        match usize::try_from(raw) {
            Ok(len) if len <= max => Ok(len),
            _ => Err(DecodeError::LengthTooLarge {
                len: u64::from(raw),
                max,
            }),
        }
    }

    /// Lit un bloc d'octets précédé de sa longueur. La disponibilité des octets est vérifiée
    /// avant toute copie : une longueur mensongère ne provoque pas d'allocation démesurée.
    pub fn get_var_bytes(&mut self, max: usize) -> Result<Vec<u8>, DecodeError> {
        let len = self.get_len(max)?;
        Ok(self.take(len)?.to_vec())
    }

    /// Vérifie qu'il ne reste aucun octet.
    pub fn finish(self) -> Result<(), DecodeError> {
        let remaining = self.remaining();
        if remaining > 0 {
            return Err(DecodeError::TrailingBytes { remaining });
        }
        Ok(())
    }
}

/// Type qui sait s'écrire sous forme canonique.
pub trait CanonicalEncode {
    /// Écrit la valeur dans `enc`.
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError>;

    /// Rend la représentation canonique complète.
    fn to_canonical_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        let mut enc = Encoder::new();
        self.encode(&mut enc)?;
        Ok(enc.into_bytes())
    }
}

/// Type qui sait se lire depuis sa forme canonique.
pub trait CanonicalDecode: Sized {
    /// Lit une valeur depuis `dec`.
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError>;

    /// Lit une valeur et refuse les octets en trop.
    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut dec = Decoder::new(bytes);
        let value = Self::decode(&mut dec)?;
        dec.finish()?;
        Ok(value)
    }
}

/// Écrit une liste : longueur bornée par `max`, puis chaque élément.
pub fn encode_vec<T: CanonicalEncode>(
    items: &[T],
    max: usize,
    enc: &mut Encoder,
) -> Result<(), EncodeError> {
    enc.put_len(items.len(), max)?;
    for item in items {
        item.encode(enc)?;
    }
    Ok(())
}

/// Lit une liste : longueur bornée par `max`, puis chaque élément.
pub fn decode_vec<T: CanonicalDecode>(
    dec: &mut Decoder<'_>,
    max: usize,
) -> Result<Vec<T>, DecodeError> {
    let len = dec.get_len(max)?;
    // La capacité est limitée par les octets restants : chaque élément occupe au moins un octet.
    let mut items = Vec::with_capacity(len.min(dec.remaining()));
    for _ in 0..len {
        items.push(T::decode(dec)?);
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_are_big_endian_fixed_width() {
        let mut enc = Encoder::new();
        enc.put_u8(0x01);
        enc.put_u32(0x0203_0405);
        enc.put_u64(0x0607_0809_0A0B_0C0D);
        assert_eq!(
            enc.into_bytes(),
            vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D]
        );
    }

    #[test]
    fn integers_roundtrip() {
        let mut enc = Encoder::new();
        enc.put_u8(255);
        enc.put_u32(u32::MAX);
        enc.put_u64(u64::MAX);
        let bytes = enc.into_bytes();
        let mut dec = Decoder::new(&bytes);
        assert_eq!(dec.get_u8().unwrap(), 255);
        assert_eq!(dec.get_u32().unwrap(), u32::MAX);
        assert_eq!(dec.get_u64().unwrap(), u64::MAX);
        assert!(dec.finish().is_ok());
    }

    #[test]
    fn truncated_input_is_rejected() {
        let mut dec = Decoder::new(&[0x00, 0x01]);
        assert_eq!(dec.get_u32(), Err(DecodeError::UnexpectedEof));
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let mut dec = Decoder::new(&[0x01, 0x02]);
        assert_eq!(dec.get_u8().unwrap(), 1);
        assert_eq!(
            dec.finish(),
            Err(DecodeError::TrailingBytes { remaining: 1 })
        );
    }

    #[test]
    fn var_bytes_roundtrip_and_bounds() {
        let mut enc = Encoder::new();
        enc.put_var_bytes(&[9, 8, 7], 10).unwrap();
        let bytes = enc.into_bytes();
        assert_eq!(bytes, vec![0, 0, 0, 3, 9, 8, 7]);
        let mut dec = Decoder::new(&bytes);
        assert_eq!(dec.get_var_bytes(10).unwrap(), vec![9, 8, 7]);

        let mut enc = Encoder::new();
        assert_eq!(
            enc.put_var_bytes(&[0; 11], 10),
            Err(EncodeError::LengthTooLarge { len: 11, max: 10 })
        );

        let mut dec = Decoder::new(&bytes);
        assert_eq!(
            dec.get_var_bytes(2),
            Err(DecodeError::LengthTooLarge { len: 3, max: 2 })
        );
    }

    #[test]
    fn lying_length_does_not_allocate_or_succeed() {
        // Annonce 1 000 000 d'octets alors qu'il n'y en a aucun.
        let bytes = [0x00, 0x0F, 0x42, 0x40];
        let mut dec = Decoder::new(&bytes);
        assert_eq!(
            dec.get_var_bytes(2_000_000),
            Err(DecodeError::UnexpectedEof)
        );
    }

    #[test]
    fn get_array_reads_exact_length() {
        let mut dec = Decoder::new(&[1, 2, 3, 4]);
        assert_eq!(dec.get_array::<3>().unwrap(), [1, 2, 3]);
        assert_eq!(dec.get_array::<2>(), Err(DecodeError::UnexpectedEof));
    }
}

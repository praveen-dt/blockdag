// src/wallet.rs

use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Serialize, Deserialize};
use serde::ser::SerializeStruct;
use hex;

#[derive(Debug)]
pub struct Wallet {
    pub keypair: Keypair,
}

impl Wallet {
    pub fn new() -> Self {
        // Manually generate secret key and derive public key
        let mut csprng = OsRng;
        let mut secret_bytes = [0u8; 32];
        csprng.fill_bytes(&mut secret_bytes);
        
        let secret_key = SecretKey::from_bytes(&secret_bytes).unwrap();
        let public_key: PublicKey = (&secret_key).into();
        let keypair = Keypair { secret: secret_key, public: public_key };

        Wallet { keypair }
    }

    pub fn get_address(&self) -> String {
        hex::encode(self.keypair.public.to_bytes())
    }

    pub fn sign(&self, message: &str) -> String {
        let signature = self.keypair.sign(message.as_bytes());
        hex::encode(signature.to_bytes())
    }

    pub fn verify(public_key: &PublicKey, message: &str, signature: &str) -> bool {
        let sig_bytes = hex::decode(signature).expect("Invalid signature hex");
        let signature = Signature::from_bytes(&sig_bytes).expect("Invalid signature bytes");
        public_key.verify(message.as_bytes(), &signature).is_ok()
    }
    pub fn load_keypair(private_key_bytes: &[u8], public_key_bytes: &[u8]) -> Keypair {
        let secret_key = SecretKey::from_bytes(private_key_bytes).expect("Invalid private key bytes");
        let public_key = PublicKey::from_bytes(public_key_bytes).expect("Invalid public key bytes");
        Keypair { secret: secret_key, public: public_key }
    }
}

// Custom serialization and deserialization for Wallet
impl Serialize for Wallet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let pub_key = hex::encode(self.keypair.public.to_bytes());
        let priv_key = hex::encode(self.keypair.secret.to_bytes());
        let mut state = serializer.serialize_struct("Wallet", 2)?;
        state.serialize_field("public_key", &pub_key)?;
        state.serialize_field("private_key", &priv_key)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Wallet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct WalletData {
            public_key: String,
            private_key: String,
        }

        let data = WalletData::deserialize(deserializer)?;
        let pub_key_bytes = hex::decode(data.public_key).map_err(serde::de::Error::custom)?;
        let priv_key_bytes = hex::decode(data.private_key).map_err(serde::de::Error::custom)?;
        let pub_key = PublicKey::from_bytes(&pub_key_bytes).map_err(serde::de::Error::custom)?;
        let secret_key = SecretKey::from_bytes(&priv_key_bytes).map_err(serde::de::Error::custom)?;
        let keypair = Keypair { public: pub_key, secret: secret_key };
        Ok(Wallet { keypair })
    }
}

use bitcoin::util::address::Payload;
use bitcoin::Script;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Ravencoin network parameters for address encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RavencoinNetwork {
    pub pubkey_hash: u8,
    pub script_hash: u8,
    pub bech32_hrp: &'static str,
}

impl RavencoinNetwork {
    pub const MAINNET: RavencoinNetwork = RavencoinNetwork {
        pubkey_hash: 60,  // 'R' prefix
        script_hash: 122, // 'r' prefix  
        bech32_hrp: "rvn",
    };
    
    pub const TESTNET: RavencoinNetwork = RavencoinNetwork {
        pubkey_hash: 111, // 'm' or 'n' prefix
        script_hash: 196, // '2' prefix
        bech32_hrp: "trvn",
    };
}

/// Ravencoin-specific script types, including asset operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RavencoinScriptType {
    // Standard Bitcoin script types
    OpReturn,
    Pay2MultiSig,
    Pay2PublicKey,
    Pay2PublicKeyHash,
    Pay2ScriptHash,
    Pay2WitnessPublicKeyHash,
    Pay2WitnessScriptHash,
    WitnessProgram,
    Unspendable,
    
    // Ravencoin-specific asset operations
    AssetIssuance,
    AssetTransfer,
    AssetReissuance,
    AssetOwnership,
    AssetQualifier,
    AssetRestricted,
    AssetMessage,
    
    NotRecognised,
}

/// Check if a script contains Ravencoin asset operations
pub fn detect_ravencoin_asset_script(script: &Script) -> Option<RavencoinScriptType> {
    let script_bytes = script.as_bytes();
    
    // Ravencoin asset scripts typically use OP_RETURN with specific prefixes
    if !script.is_op_return() || script_bytes.len() < 3 {
        return None;
    }
    
    // Skip OP_RETURN (0x6a) and length byte
    let data = &script_bytes[2..];
    
    if data.is_empty() {
        return None;
    }
    
    // Check for Ravencoin asset operation markers
    match data.get(0) {
        Some(0x72) => { // 'r' - Asset operations
            if data.len() >= 4 {
                match &data[1..4] {
                    b"vnq" => Some(RavencoinScriptType::AssetIssuance),
                    b"vnt" => Some(RavencoinScriptType::AssetTransfer),
                    b"vnr" => Some(RavencoinScriptType::AssetReissuance),
                    b"vno" => Some(RavencoinScriptType::AssetOwnership),
                    b"vnQ" => Some(RavencoinScriptType::AssetQualifier),
                    b"vnR" => Some(RavencoinScriptType::AssetRestricted),
                    b"vnm" => Some(RavencoinScriptType::AssetMessage),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Create a Ravencoin address from a Bitcoin address payload using proper network params
pub fn ravencoin_address_from_payload(payload: &Payload, network: RavencoinNetwork) -> Option<String> {
    match payload {
        Payload::PubkeyHash(pkh) => {
            // For mainnet Ravencoin, this should produce addresses starting with 'R'
            let mut bytes = vec![network.pubkey_hash];
            bytes.extend_from_slice(&pkh[..]);
            Some(encode_base58_check(&bytes))
        }
        Payload::ScriptHash(sh) => {
            // For mainnet Ravencoin, this should produce addresses starting with 'r' 
            let mut bytes = vec![network.script_hash];
            bytes.extend_from_slice(&sh[..]);
            Some(encode_base58_check(&bytes))
        }
        Payload::WitnessProgram { version: _, program } => {
            // For Ravencoin bech32 encoding with 'rvn' hrp
            encode_bech32(network.bech32_hrp, program).ok()
        }
    }
}

/// Proper Base58Check encoding implementation
fn encode_base58_check(data: &[u8]) -> String {
    // Calculate double SHA256 checksum
    let first_hash = Sha256::digest(data);
    let second_hash = Sha256::digest(&first_hash);
    let checksum = &second_hash[0..4];
    
    // Combine data + checksum
    let mut full_data = Vec::with_capacity(data.len() + 4);
    full_data.extend_from_slice(data);
    full_data.extend_from_slice(checksum);
    
    // Encode with Base58
    bs58::encode(full_data).into_string()
}

/// Simple bech32 encoding (placeholder implementation)
fn encode_bech32(hrp: &str, data: &[u8]) -> Result<String, &'static str> {
    // Placeholder - in production use proper bech32 crate
    // For now, just return a basic format since Ravencoin doesn't widely use bech32 yet
    Ok(format!("{}1{}", hrp, bs58::encode(data).into_string().to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::hex::FromHex;
    
    #[test]
    fn test_ravencoin_asset_detection() {
        // Test asset issuance script (OP_RETURN + length + "rvnq" + asset data)
        let script_hex = "6a0472766e71"; // OP_RETURN(6a) + len(04) + "rvnq"
        let script = Script::from_hex(script_hex).unwrap();
        
        assert_eq!(
            detect_ravencoin_asset_script(&script),
            Some(RavencoinScriptType::AssetIssuance)
        );
    }
    
    #[test] 
    fn test_ravencoin_network_constants() {
        assert_eq!(RavencoinNetwork::MAINNET.pubkey_hash, 60);
        assert_eq!(RavencoinNetwork::MAINNET.script_hash, 122);
        assert_eq!(RavencoinNetwork::MAINNET.bech32_hrp, "rvn");
    }
    
    #[test]
    fn test_standard_bitcoin_script_not_detected_as_asset() {
        // Standard P2PKH script from existing tests (even length hex)
        let script_hex = "76a91412ab8dc588ca9d5787dde7eb29569da63c3a238c88ac";
        let script = Script::from_hex(script_hex).unwrap();
        
        assert_eq!(detect_ravencoin_asset_script(&script), None);
    }
    
    #[test]
    fn test_ravencoin_address_encoding() {
        use bitcoin::{PubkeyHash, ScriptHash};
        use bitcoin::hashes::{hash160, Hash};
        
        // Test with a known hash160 (20 bytes)
        let test_hash = [
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
            0x12, 0x34, 0x56, 0x78
        ];
        
        let network = RavencoinNetwork::MAINNET;
        
        // Test P2PKH address generation
        let hash160_obj = hash160::Hash::from_slice(&test_hash).unwrap();
        let pkh = PubkeyHash::from_hash(hash160_obj);
        let p2pkh_payload = Payload::PubkeyHash(pkh);
        let p2pkh_addr = ravencoin_address_from_payload(&p2pkh_payload, network).unwrap();
        assert!(p2pkh_addr.starts_with('R')); // Ravencoin mainnet P2PKH addresses start with 'R'
        
        // Test P2SH address generation  
        let hash160_obj = hash160::Hash::from_slice(&test_hash).unwrap();
        let sh = ScriptHash::from_hash(hash160_obj);
        let p2sh_payload = Payload::ScriptHash(sh);
        let p2sh_addr = ravencoin_address_from_payload(&p2sh_payload, network).unwrap();
        assert!(p2sh_addr.starts_with('r')); // Ravencoin mainnet P2SH addresses start with 'r'
        
        println!("P2PKH: {}", p2pkh_addr);
        println!("P2SH: {}", p2sh_addr);
    }
}

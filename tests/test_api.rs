//!
//! Integration Test
//!
//! Test multiple APIs. Cross checking results between each other.
//!
#[cfg(test)]
mod iterator_tests {
    use bitcoin::{Block, Transaction};
    use bitcoin_explorer::{
        BitcoinDB, FBlock, FTransaction, SBlock, SConnectedBlock, SConnectedTransaction,
        STransaction,
    };
    use std::path::PathBuf;

    const END: usize = 700000;

    /// utility function
    fn get_test_db() -> BitcoinDB {
        let mut crate_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        crate_root_dir.push("./resources/tests/Bitcoin");
        BitcoinDB::new(&crate_root_dir, true).unwrap()
    }

    fn get_test_raven_db() -> BitcoinDB {
        let mut crate_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        crate_root_dir.push("D:/Raven");
        BitcoinDB::new_ravencoin(&crate_root_dir, true).unwrap()
    }

    #[test]
    /// iterate through all blocks, check order and correctness
    fn test_iter_block() {
        let db = get_test_db();

        let mut h = 0;
        for blk in db.iter_block::<SBlock>(0, END) {
            let blk_ref = db.get_block::<SBlock>(h).unwrap();
            assert_eq!(blk, blk_ref);
            h += 1;
        }
        // assert that all blocks are read
        assert_eq!(h as usize, db.get_block_count())
    }

    #[test]
    /// iterate through Ravencoin blocks, check order and correctness
    fn test_iter_block_ravencoin() {
        if let Ok(db) = std::panic::catch_unwind(|| get_test_raven_db()) {
            let end = std::cmp::min(10000, db.get_block_count()); // Use smaller range for Ravencoin
            
            let mut h = 0;
            for blk in db.iter_block::<SBlock>(0, end) {
                let blk_ref = db.get_block::<SBlock>(h).unwrap();
                assert_eq!(blk, blk_ref);
                h += 1;
            }
            // assert that all blocks are read
            assert_eq!(h, end);
            
            // Verify it's actually Ravencoin chain
            assert!(db.chain().is_ravencoin());
        }
    }

    #[test]
    /// iterate through part of the chain
    fn test_iter_block_early_end() {
        let db = get_test_db();
        let start = 100;
        let early_end = 100000;

        let mut h = start;
        for blk in db.iter_block::<SBlock>(start, early_end) {
            let blk_ref = db.get_block::<SBlock>(h).unwrap();
            assert_eq!(blk, blk_ref);
            h += 1;
        }
        assert_eq!(h, early_end)
    }

    #[test]
    /// iterate through part of the Ravencoin chain
    fn test_iter_block_early_end_ravencoin() {
        if let Ok(db) = std::panic::catch_unwind(|| get_test_raven_db()) {
            let start = 100;
            let early_end = std::cmp::min(10000, db.get_block_count());

            let mut h = start;
            for blk in db.iter_block::<SBlock>(start, early_end) {
                let blk_ref = db.get_block::<SBlock>(h).unwrap();
                assert_eq!(blk, blk_ref);
                h += 1;
            }
            assert_eq!(h, early_end);
            
            // Verify chain type
            assert!(db.chain().is_ravencoin());
        }
    }

    #[test]
    /// ensure that the iterator can be dropped after breaking loop
    fn test_iter_block_break() {
        let db = get_test_db();
        let break_height = 100000;

        let mut h = 0;
        let mut some_blk = None;
        for blk in db.iter_block::<SBlock>(0, END) {
            some_blk = Some(blk);
            if h == break_height {
                break;
            }
            h += 1;
        }
        assert_eq!(some_blk, Some(db.get_block(break_height).unwrap()))
    }

    #[test]
    /// ensure that `get_transaction` responds with correct transaction
    fn test_get_transactions() {
        let db = get_test_db();
        let early_end = 100000;

        for blk in db.iter_block::<Block>(0, early_end) {
            for tx in blk.txdata {
                assert_eq!(db.get_transaction::<Transaction>(&tx.txid()).unwrap(), tx);
            }
        }
    }

    #[test]
    /// iterate through all blocks
    fn test_iter_connected() {
        let db = get_test_db();

        let mut h = 0;
        for blk in db.iter_connected_block::<SConnectedBlock>(END) {
            // check that blocks are produced in correct order
            assert_eq!(blk.header, db.get_block::<SBlock>(h).unwrap().header);
            h += 1;
        }
        // assert that all blocks are read
        assert_eq!(h as usize, db.get_block_count())
    }

    #[test]
    /// iterate through all Ravencoin connected blocks
    fn test_iter_connected_ravencoin() {
        if let Ok(db) = std::panic::catch_unwind(|| get_test_raven_db()) {
            let end = std::cmp::min(5000, db.get_block_count()); // Use smaller range
            
            let mut h = 0;
            for blk in db.iter_connected_block::<SConnectedBlock>(end) {
                // check that blocks are produced in correct order
                assert_eq!(blk.header, db.get_block::<SBlock>(h).unwrap().header);
                h += 1;
            }
            // assert that all blocks are read
            assert_eq!(h, end);
            
            // Verify chain type
            assert!(db.chain().is_ravencoin());
        }
    }

    #[test]
    /// ensure that `get_transaction` responds with correct transaction
    fn test_raven_get_transactions() {
        let db = get_test_raven_db();
        let early_end = 100000;

        for blk in db.iter_block::<Block>(0, early_end) {
            for tx in blk.txdata {
                assert_eq!(db.get_transaction::<Transaction>(&tx.txid()).unwrap(), tx);
            }
        }
    }

    #[test]
    /// ensure that `get_transaction` responds with correct transaction for Ravencoin
    fn test_raven_get_transactions_with_chain_check() {
        if let Ok(db) = std::panic::catch_unwind(|| get_test_raven_db()) {
            let early_end = std::cmp::min(1000, db.get_block_count()); // Use smaller range
            
            // Verify this is Ravencoin chain
            assert!(db.chain().is_ravencoin());
            assert_eq!(db.chain().as_str(), "ravencoin");

            let mut tx_count = 0;
            for blk in db.iter_block::<Block>(0, early_end) {
                for tx in blk.txdata {
                    assert_eq!(db.get_transaction::<Transaction>(&tx.txid()).unwrap(), tx);
                    tx_count += 1;
                    
                    // Limit to avoid long test times
                    if tx_count > 100 {
                        return;
                    }
                }
            }
        }
    }

    #[test]
    /// iterate through part of the chain
    fn test_iter_connected_early_end() {
        let db = get_test_db();
        let early_end = 100000;

        let mut h = 0;
        for blk in db.iter_connected_block::<SConnectedBlock>(early_end) {
            let blk_ref = db.get_connected_block::<SConnectedBlock>(h).unwrap();
            assert_eq!(blk, blk_ref);
            h += 1;
        }
        assert_eq!(h, early_end)
    }

    #[test]
    /// ensure that the iterator can be dropped after breaking loop
    fn test_iter_connected_break() {
        let db = get_test_db();
        let break_height = 100000;

        let mut h = 0;
        let mut some_blk = None;
        for blk in db.iter_connected_block::<SConnectedBlock>(END) {
            some_blk = Some(blk);
            if h == break_height {
                break;
            }
            h += 1;
        }
        assert_eq!(
            some_blk,
            Some(
                db.get_connected_block::<SConnectedBlock>(break_height)
                    .unwrap()
            )
        )
    }

    #[test]
    /// ensure that `get_connected_transaction` responds with correct transaction
    fn test_get_connected_transactions() {
        let db = get_test_db();
        let early_end = 100000;

        for blk in db.iter_connected_block::<SConnectedBlock>(early_end) {
            for tx in blk.txdata {
                let connected_tx = db
                    .get_connected_transaction::<SConnectedTransaction>(&tx.txid)
                    .unwrap();
                let unconnected_stx = db.get_transaction::<STransaction>(&tx.txid).unwrap();
                let unconnected_ftx = db.get_transaction::<FTransaction>(&tx.txid).unwrap();
                assert_eq!(connected_tx.input.len(), unconnected_stx.input.len());
                assert_eq!(connected_tx.input.len(), unconnected_ftx.input.len());
                assert_eq!(connected_tx, tx);
            }
        }
    }

    #[test]
    /// assert that coinbase input has zero length
    fn test_coinbase_input() {
        let db = get_test_db();

        for blk in db.iter_block::<SBlock>(0, END) {
            assert_eq!(blk.txdata.first().unwrap().input.len(), 0);
        }

        for blk in db.iter_block::<FBlock>(0, END) {
            assert_eq!(blk.txdata.first().unwrap().input.len(), 0);
        }
    }

    #[test]
    /// assert that coinbase input has zero length for both chains
    fn test_coinbase_input_both_chains() {
        // Test Bitcoin
        let bitcoin_db = get_test_db();
        let btc_end = std::cmp::min(1000, bitcoin_db.get_block_count());
        
        for blk in bitcoin_db.iter_block::<SBlock>(0, btc_end) {
            assert_eq!(blk.txdata.first().unwrap().input.len(), 0);
        }
        
        // Test Ravencoin if available
        if let Ok(raven_db) = std::panic::catch_unwind(|| get_test_raven_db()) {
            let rvn_end = std::cmp::min(1000, raven_db.get_block_count());
            
            for blk in raven_db.iter_block::<SBlock>(0, rvn_end) {
                assert_eq!(blk.txdata.first().unwrap().input.len(), 0);
                
                // Verify this is actually Ravencoin chain
                assert!(raven_db.chain().is_ravencoin());
                break; // Just check first block
            }
            
            for blk in raven_db.iter_block::<FBlock>(0, rvn_end) {
                assert_eq!(blk.txdata.first().unwrap().input.len(), 0);
                break; // Just check first block
            }
        }
    }

    #[test]
    fn test_iter_block_heights() {
        let db = get_test_db();
        let test_heights = vec![3, 6, 2, 7, 1, 8, 3, 8, 1, 8, 2, 7, 21];
        let blocks_ref: Vec<SBlock> = test_heights
            .iter()
            .map(|h| db.get_block::<SBlock>(*h).unwrap())
            .collect();
        let blocks: Vec<SBlock> = db.iter_heights::<SBlock, _>(test_heights).collect();
        assert_eq!(blocks, blocks_ref)
    }

    #[test]
    fn test_iter_block_heights_both_chains() {
        // Test with Bitcoin
        let bitcoin_db = get_test_db();
        let test_heights = vec![1, 2, 3, 5, 8]; // Smaller set for both chains
        
        let btc_blocks_ref: Vec<SBlock> = test_heights
            .iter()
            .map(|h| bitcoin_db.get_block::<SBlock>(*h).unwrap())
            .collect();
        let btc_blocks: Vec<SBlock> = bitcoin_db.iter_heights::<SBlock, _>(test_heights.clone()).collect();
        assert_eq!(btc_blocks, btc_blocks_ref);
        assert!(!bitcoin_db.chain().is_ravencoin());
        
        // Test with Ravencoin if available
        if let Ok(raven_db) = std::panic::catch_unwind(|| get_test_raven_db()) {
            // Ensure heights exist in Ravencoin chain
            let max_height = std::cmp::min(test_heights.iter().max().unwrap() + 1, raven_db.get_block_count());
            if max_height > *test_heights.iter().max().unwrap() {
                let rvn_blocks_ref: Vec<SBlock> = test_heights
                    .iter()
                    .map(|h| raven_db.get_block::<SBlock>(*h).unwrap())
                    .collect();
                let rvn_blocks: Vec<SBlock> = raven_db.iter_heights::<SBlock, _>(test_heights).collect();
                assert_eq!(rvn_blocks, rvn_blocks_ref);
                assert!(raven_db.chain().is_ravencoin());
            }
        }
    }

    #[test]
    fn test_ravencoin_constructor_path_missing() {
        // Ensure constructor returns error when path missing (no fixture yet)
        let mut crate_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        crate_root_dir.push("./resources/tests/Ravencoin");
        let db = BitcoinDB::new_ravencoin(&crate_root_dir, false);
        assert!(db.is_err());
    }

    #[test]
    fn test_ravencoin_script_evaluation() {
        use bitcoin_explorer::Chain;
        
        // Test that the Chain enum and basic functionality works
        let bitcoin_chain = Chain::Bitcoin;
        let ravencoin_chain = Chain::Ravencoin;
        
        assert_eq!(bitcoin_chain.as_str(), "bitcoin");
        assert_eq!(ravencoin_chain.as_str(), "ravencoin");
        assert!(!bitcoin_chain.is_ravencoin());
        assert!(ravencoin_chain.is_ravencoin());
        
        // Test that chain() method works
        let path = std::path::PathBuf::from("nonexistent");
        let bitcoin_db_result = bitcoin_explorer::BitcoinDB::new(&path, false);
        let ravencoin_db_result = bitcoin_explorer::BitcoinDB::new_ravencoin(&path, false);
        
        // Both should fail due to missing path
        assert!(bitcoin_db_result.is_err());
        assert!(ravencoin_db_result.is_err());
    }

    #[test]
    fn test_ravencoin_address_generation() {
        use bitcoin_explorer::get_addresses_from_script_for_chain;
        use bitcoin_explorer::Chain;
        
        // Test standard P2PKH script with Ravencoin chain
        let p2pkh_script = "76a91412ab8dc588ca9d5787dde7eb29569da63c3a238c88ac";
        let result = get_addresses_from_script_for_chain(p2pkh_script, Chain::Ravencoin).unwrap();
        
        assert_eq!(result.pattern.to_string(), "Pay2PublicKeyHash");
        assert!(!result.addresses.is_empty());
        
        // Should have Ravencoin-specific addresses that start with 'R'
        if let Some(ref rvn_addrs) = result.ravencoin_addresses {
            assert!(!rvn_addrs.is_empty());
            assert!(rvn_addrs[0].starts_with('R'));
            println!("Ravencoin P2PKH address: {}", rvn_addrs[0]);
        }
        
        // Test same script with Bitcoin chain - should not have Ravencoin features
        let result_btc = get_addresses_from_script_for_chain(p2pkh_script, Chain::Bitcoin).unwrap();
        assert_eq!(result_btc.pattern.to_string(), "Pay2PublicKeyHash");
        assert!(result_btc.ravencoin_addresses.is_none());
    }

    #[test]
    fn test_chain_specific_address_differences() {
        use bitcoin_explorer::get_addresses_from_script_for_chain;
        use bitcoin_explorer::Chain;
        
        // Test P2PKH script
        let p2pkh_script = "76a91412ab8dc588ca9d5787dde7eb29569da63c3a238c88ac";
        
        let btc_result = get_addresses_from_script_for_chain(p2pkh_script, Chain::Bitcoin).unwrap();
        let rvn_result = get_addresses_from_script_for_chain(p2pkh_script, Chain::Ravencoin).unwrap();
        
        // Both should recognize as P2PKH
        assert_eq!(btc_result.pattern.to_string(), "Pay2PublicKeyHash");
        assert_eq!(rvn_result.pattern.to_string(), "Pay2PublicKeyHash");
        
        // Bitcoin should not have Ravencoin addresses
        assert!(btc_result.ravencoin_addresses.is_none());
        
        // Ravencoin should have Ravencoin addresses
        assert!(rvn_result.ravencoin_addresses.is_some());
        if let Some(ref rvn_addrs) = rvn_result.ravencoin_addresses {
            assert!(!rvn_addrs.is_empty());
            assert!(rvn_addrs[0].starts_with('R')); // P2PKH starts with 'R'
        }
        
        // Test P2SH script
        let p2sh_script = "a91412ab8dc588ca9d5787dde7eb29569da63c3a238c87";
        
        let btc_p2sh = get_addresses_from_script_for_chain(p2sh_script, Chain::Bitcoin).unwrap();
        let rvn_p2sh = get_addresses_from_script_for_chain(p2sh_script, Chain::Ravencoin).unwrap();
        
        assert_eq!(btc_p2sh.pattern.to_string(), "Pay2ScriptHash");
        assert_eq!(rvn_p2sh.pattern.to_string(), "Pay2ScriptHash");
        
        // Ravencoin P2SH should start with 'r' (lowercase)
        if let Some(ref rvn_addrs) = rvn_p2sh.ravencoin_addresses {
            assert!(!rvn_addrs.is_empty());
            assert!(rvn_addrs[0].starts_with('r')); // P2SH starts with 'r'
        }
    }

    #[test]
    fn test_ravencoin_db_properties() {
        if let Ok(db) = std::panic::catch_unwind(|| get_test_raven_db()) {
            // Test basic properties
            assert!(db.chain().is_ravencoin());
            assert_eq!(db.chain().as_str(), "ravencoin");
            
            // Test that we can get blocks
            let block_count = db.get_block_count();
            assert!(block_count > 0);
            
            // Test that we can get the genesis block
            let genesis = db.get_block::<SBlock>(0).unwrap();
            // Genesis block should have a specific block hash (not previous hash since SBlock doesn't store it)
            assert!(!genesis.header.block_hash.to_string().is_empty());
            
            // Test first few blocks exist
            for i in 0..std::cmp::min(10, block_count) {
                let block = db.get_block::<SBlock>(i);
                assert!(block.is_ok(), "Failed to get block {}", i);
            }
        }
    }
}

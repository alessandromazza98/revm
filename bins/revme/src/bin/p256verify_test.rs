use hex_literal::hex;
use revm::{
    inspectors::TracerEip3155,
    primitives::{Env, SpecId::LATEST, TransactTo, B160, KECCAK_EMPTY, U256},
};
use std::io::stdout;
use std::str::FromStr;

fn main() {
    // Create database and insert cache
    let mut cache_state = revm::CacheState::new(false);
    let p256verify_address = B160::from_str("0x000000000000000000000000000000000000000a").unwrap();
    let sender_address = B160::from_str("0xa000000000000000000000000000000000000000").unwrap();
    let acc_info = revm::primitives::AccountInfo {
        balance: U256::from(100_000_000),
        code_hash: KECCAK_EMPTY,
        code: None,
        nonce: 1,
    };
    cache_state.insert_account(sender_address, acc_info);

    // create env with default values
    let mut env = Env::default();

    // cfg env
    env.cfg.chain_id = 1; // for mainnet
    env.cfg.spec_id = LATEST;

    //tx env
    env.tx.caller = sender_address;
    env.tx.nonce = Some(1);
    env.tx.gas_price = U256::from(1);
    env.tx.transact_to = TransactTo::Call(p256verify_address);
    env.tx.gas_limit = 50_000;
    env.tx.data = hex!("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d604aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e").to_vec().into();

    // create EVM
    let cache = cache_state.clone();
    let mut state = revm::db::State::builder()
        .with_cached_prestate(cache)
        .with_bundle_update()
        .build();

    let mut evm = revm::new();
    evm.database(&mut state);
    evm.env = env.clone();

    // execute tx
    let result = evm
        .inspect_commit(TracerEip3155::new(Box::new(stdout()), false, false))
        .unwrap();

    println!("{:?}", result);
}

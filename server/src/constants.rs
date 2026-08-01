use std::{env, sync::LazyLock};

#[cfg(not(feature = "dev"))]
pub static FULLCHAIN_PATH: LazyLock<String> =
    LazyLock::new(|| env::var("FULLCHAIN_PATH").expect("FULLCHAIN_PATH must be set"));

#[cfg(not(feature = "dev"))]
pub static PRIVKEY_PATH: LazyLock<String> =
    LazyLock::new(|| env::var("PRIVKEY_PATH").expect("PRIVKEY_PATH must be set"));

/// Where the PnL ledger and hand log live. Unlike the certificate paths this
/// has a working default, so a dev server needs no extra setup.
pub static DATA_DIR: LazyLock<String> =
    LazyLock::new(|| env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string()));

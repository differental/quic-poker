use std::{env, sync::LazyLock};

pub static FULLCHAIN_PATH: LazyLock<String> =
    LazyLock::new(|| env::var("FULLCHAIN_PATH").expect("FULLCHAIN_PATH must be set"));

pub static PRIVKEY_PATH: LazyLock<String> =
    LazyLock::new(|| env::var("PRIVKEY_PATH").expect("PRIVKEY_PATH must be set"));

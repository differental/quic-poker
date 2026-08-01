//! Durable storage for the PnL ledger.
//!
//! Two files under [`DATA_DIR`]:
//!
//! - `accounts.json` — the running ledger, rewritten after each hand
//! - `hands.jsonl` — one JSON object per settled hand, appended and never
//!   rewritten. This is the record the ledger can be rebuilt from.
//!
//! Accounts are keyed by player name rather than by `PlayerId`, because a
//! `PlayerId` is minted per connection and restarts at 0 on every boot — a file
//! keyed on one would hand a brand-new player the previous run's balance.
//!
//! Every operation here is best-effort: a storage failure is logged and the
//! game carries on rather than taking the server down mid-hand.

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::{error, info};

use crate::constants::DATA_DIR;

/// A player's persistent record, keyed by name.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Account {
    /// Running profit or loss across every hand this account has ever played.
    pub pnl: i64,
    /// Everything ever bought in for, including rebuys. A rebuy is deliberately
    /// not a PnL event — it adds chips and an equal debt — which is what keeps
    /// `pnl == stack - total_bought_in` true.
    pub total_bought_in: u64,
    pub hands_played: u64,
}

/// One player's line in a hand-log record.
pub struct HandLogPlayer {
    pub name: String,
    pub pnl: i64,
    pub stack_after: u64,
}

#[derive(Serialize)]
struct HandLogPlayerRecord<'a> {
    name: &'a str,
    pnl: i64,
    stack_after: u64,
}

#[derive(Serialize)]
struct HandLogRecord<'a> {
    ts: u64,
    table: u32,
    pot: u64,
    players: Vec<HandLogPlayerRecord<'a>>,
}

fn accounts_path() -> PathBuf {
    PathBuf::from(&*DATA_DIR).join("accounts.json")
}

fn hands_path() -> PathBuf {
    PathBuf::from(&*DATA_DIR).join("hands.jsonl")
}

/// Reads the ledger from disk. A missing file is a normal first start, not an
/// error. A *corrupt* file is an error, and we start empty rather than refuse to
/// boot — but it is logged loudly, since it means losing the running totals.
pub fn load_accounts() -> HashMap<String, Account> {
    let path = accounts_path();
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!(path = %path.display(), "No ledger yet, starting empty");
            return HashMap::new();
        }
        Err(e) => {
            error!(path = %path.display(), error = %e, "Failed to read ledger, starting empty");
            return HashMap::new();
        }
    };

    match serde_json::from_str(&raw) {
        Ok(accounts) => accounts,
        Err(e) => {
            error!(path = %path.display(), error = %e, "Ledger is corrupt, starting empty");
            HashMap::new()
        }
    }
}

/// Writes the ledger out by rendering to a temporary file and renaming over the
/// target, so a crash part-way through cannot truncate the existing ledger.
pub fn save_accounts(accounts: &HashMap<String, Account>) {
    if let Err(e) = fs::create_dir_all(&*DATA_DIR) {
        error!(dir = %*DATA_DIR, error = %e, "Failed to create data directory");
        return;
    }

    let path = accounts_path();
    let tmp = path.with_extension("json.tmp");

    let encoded = match serde_json::to_vec_pretty(accounts) {
        Ok(encoded) => encoded,
        Err(e) => {
            error!(error = %e, "Failed to encode ledger");
            return;
        }
    };

    if let Err(e) = fs::write(&tmp, &encoded) {
        error!(path = %tmp.display(), error = %e, "Failed to write ledger");
        return;
    }
    if let Err(e) = fs::rename(&tmp, &path) {
        error!(path = %path.display(), error = %e, "Failed to commit ledger");
    }
}

/// Appends one settled hand to the hand log.
pub fn append_hand(table: u32, pot: u64, players: &[HandLogPlayer]) {
    if let Err(e) = fs::create_dir_all(&*DATA_DIR) {
        error!(dir = %*DATA_DIR, error = %e, "Failed to create data directory");
        return;
    }

    let record = HandLogRecord {
        ts: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        table,
        pot,
        players: players
            .iter()
            .map(|p| HandLogPlayerRecord {
                name: &p.name,
                pnl: p.pnl,
                stack_after: p.stack_after,
            })
            .collect(),
    };

    let mut line = match serde_json::to_string(&record) {
        Ok(line) => line,
        Err(e) => {
            error!(error = %e, "Failed to encode hand log entry");
            return;
        }
    };
    line.push('\n');

    let path = hands_path();
    let appended = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut file| file.write_all(line.as_bytes()));

    if let Err(e) = appended {
        error!(path = %path.display(), error = %e, "Failed to append to hand log");
    }
}

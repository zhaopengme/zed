pub mod kvp;
pub mod workspace;

use std::fs;
use std::ops::Deref;
use std::path::Path;

use anyhow::Result;
use indoc::indoc;
use kvp::KVP_MIGRATION;
use sqlez::connection::Connection;
use sqlez::thread_safe_connection::ThreadSafeConnection;
use workspace::items::ITEM_MIGRATIONS;
use workspace::pane::PANE_MIGRATIONS;

pub use workspace::*;

#[derive(Clone)]
pub struct Db(ThreadSafeConnection);

impl Deref for Db {
    type Target = sqlez::connection::Connection;

    fn deref(&self) -> &Self::Target {
        &self.0.deref()
    }
}

impl Db {
    /// Open or create a database at the given directory path.
    pub fn open(db_dir: &Path, channel: &'static str) -> Self {
        // Use 0 for now. Will implement incrementing and clearing of old db files soon TM
        let current_db_dir = db_dir.join(Path::new(&format!("0-{}", channel)));
        fs::create_dir_all(&current_db_dir)
            .expect("Should be able to create the database directory");
        let db_path = current_db_dir.join(Path::new("db.sqlite"));

        Db(initialize_connection(ThreadSafeConnection::new(
            db_path.to_string_lossy().as_ref(),
            true,
        )))
    }

    /// Open a in memory database for testing and as a fallback.
    pub fn open_in_memory(db_name: &str) -> Self {
        Db(initialize_connection(ThreadSafeConnection::new(
            db_name, false,
        )))
    }

    pub fn persisting(&self) -> bool {
        self.persistent()
    }

    pub fn write_to<P: AsRef<Path>>(&self, dest: P) -> Result<()> {
        let destination = Connection::open_file(dest.as_ref().to_string_lossy().as_ref());
        self.backup_main(&destination)
    }
}

fn initialize_connection(conn: ThreadSafeConnection) -> ThreadSafeConnection {
    conn.with_initialize_query(indoc! {"
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        PRAGMA foreign_keys=TRUE;
        PRAGMA case_sensitive_like=TRUE;
        "})
        .with_migrations(&[
            KVP_MIGRATION,
            WORKSPACES_MIGRATION,
            PANE_MIGRATIONS,
            ITEM_MIGRATIONS,
        ])
}

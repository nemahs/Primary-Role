use serenity::all::{GuildId, RoleId};
use sqlite;
use sqlite::State;

pub struct AppData {
    db: sqlite::Connection,
}

type SQLResult = Result<(), sqlite::Error>;

impl AppData {
    pub fn new() -> Self {
        let new_db = Self {
            db: sqlite::Connection::open(":memory:")
                .expect("Expected in memory database to initialize"),
        };

        new_db
            .db
            .execute(
                "CREATE TABLE IF NOT EXISTS roles (
          guild_id INTEGER PRIMARY KEY,
          role_id INTEGER,
          auto_scan BOOLEAN DEFAULT(TRUE)
        );",
            )
            .expect("Table was unable to be created.");

        new_db
    }

    pub fn new_server(&mut self, server_id: &GuildId) -> SQLResult {
        let statement = format!(
            "INSERT OR IGNORE INTO roles (guild_id, role_id) VALUES({}, NULL);",
            server_id.get()
        );
        self.db.execute(statement)
    }

    pub fn update_server_primary_role(
        &mut self,
        server_id: &GuildId,
        role_id: &RoleId,
    ) -> SQLResult {
        let statement = format!(
            "UPDATE OR IGNORE roles SET role_id = {} WHERE guild_id = {};",
            role_id.get(),
            server_id.get()
        );

        self.db.execute(statement)
    }

    pub fn is_auto_scan_enabled(&self, server_id: &GuildId) -> bool {
        let statement = format!(
            "SELECT auto_scan FROM roles WHERE guild_id = {};",
            server_id.get()
        );
        let mut statement = self.db.prepare(statement).unwrap();

        while let Ok(State::Row) = statement.next() {
            return statement
                .read::<i64, _>("auto_scan")
                .map_or(false, |val| val != 0);
        }

        return false;
    }

    pub fn disable_auto_scan(&self, server_id: &GuildId) -> SQLResult {
        let statement = format!(
            "UPDATE OR IGNORE roles SET auto_scan = FALSE WHERE guild_id = {}",
            server_id.get()
        );

        self.db.execute(statement)
    }

    pub fn enable_auto_scan(&self, server_id: &GuildId) -> SQLResult {
        let statement = format!(
            "UPDATE OR IGNORE roles SET auto_scan = TRUE WHERE guild_id = {}",
            server_id.get()
        );

        self.db.execute(statement)
    }

    pub fn get_primary_role(&self, server_id: &GuildId) -> Option<RoleId> {
        let statement = format!(
            "SELECT role_id FROM roles where guild_id = {};",
            server_id.get()
        );
        let mut statement = self.db.prepare(statement).ok()?;

        while let Ok(State::Row) = statement.next() {
            let id = statement.read::<i64, _>("role_id").ok()?;
            let id: u64 = id.try_into().ok()?;

            return Some(RoleId::new(id));
        }

        return None;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_auto_scan() {
        let mut test_subject = AppData::new();
        let guild1 = GuildId::new(1);
        let guild2 = GuildId::new(2);

        test_subject.new_server(&guild1).unwrap();
        test_subject.new_server(&guild2).unwrap();
        test_subject.disable_auto_scan(&guild2).unwrap();

        assert_eq!(true, test_subject.is_auto_scan_enabled(&guild1));
        assert_eq!(false, test_subject.is_auto_scan_enabled(&guild2));
        assert_eq!(false, test_subject.is_auto_scan_enabled(&GuildId::new(37)));
    }
}

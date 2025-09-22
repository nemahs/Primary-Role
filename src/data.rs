use log::error;
use serenity::all::{GuildId, RoleId};
use sqlite;
use sqlite::State;

pub struct AppData {
    db: sqlite::Connection,
}

type SQLResult = Result<(), sqlite::Error>;

impl AppData {
    pub fn new(db_location: &str) -> Self {
        let new_db = Self {
            db: sqlite::Connection::open(db_location).expect("Expected the database to initialize"),
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

    /// Register a new server with the database
    ///
    /// @param server_id ID of the new server
    pub fn new_server(&mut self, server_id: &GuildId) -> SQLResult {
        let statement = format!(
            "INSERT OR IGNORE INTO roles (guild_id, role_id) VALUES({}, NULL);",
            server_id.get()
        );
        self.db.execute(statement)
    }

    /// Update the primary role for the given server
    ///
    /// @param server_id ID for the server to update
    /// @param role_id ID to become the new primary role
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

    /// Get if auto scanning is enabled for the given server
    ///
    /// @param server_id ID of the server to check
    pub fn is_auto_scan_enabled(&self, server_id: &GuildId) -> bool {
        let statement = format!(
            "SELECT auto_scan FROM roles WHERE guild_id = {};",
            server_id.get()
        );
        let statement = self.db.prepare(statement);
        let Ok(mut statement) = statement else {
            error!("Failed to prepare statement");
            return false;
        };

        while let Ok(State::Row) = statement.next() {
            return statement
                .read::<i64, _>("auto_scan")
                .map_or(false, |val| val != 0);
        }

        return false;
    }

    /// Disable auto scan on a given server
    ///
    /// @param server_id ID of the server to disable auto scanning on
    pub fn disable_auto_scan(&self, server_id: &GuildId) -> SQLResult {
        let statement = format!(
            "UPDATE OR IGNORE roles SET auto_scan = FALSE WHERE guild_id = {}",
            server_id.get()
        );

        self.db.execute(statement)
    }

    /// Enable auto scan on a given server
    ///
    /// @param server_id ID of the server to enable auto scanning on
    pub fn enable_auto_scan(&self, server_id: &GuildId) -> SQLResult {
        let statement = format!(
            "UPDATE OR IGNORE roles SET auto_scan = TRUE WHERE guild_id = {}",
            server_id.get()
        );

        self.db.execute(statement)
    }

    /// Get the primary role for a given server
    ///
    /// @param server_id ID for the server to get the primary role of
    ///
    /// @return Role ID of the primary role, or None if not saved
    pub fn get_primary_role(&self, server_id: &GuildId) -> Option<RoleId> {
        let statement = format!(
            "SELECT role_id FROM roles where guild_id = {};",
            server_id.get()
        );
        let mut statement = self.db.prepare(statement).ok()?;

        while let Ok(State::Row) = statement.next() {
            let id = statement.read::<i64, _>("role_id").ok()?;
            let id: u64 = id.try_into().ok()?;

            if id == 0 {
                return None;
            }

            return Some(RoleId::new(id));
        }

        return None;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::Config;
    use proptest_state_machine::{
        self, prop_state_machine, ReferenceStateMachine, StateMachineTest,
    };

    // Normal tests

    #[test]
    fn test_auto_scan() {
        let mut test_subject = AppData::new(":memory:");
        let guild1 = GuildId::new(1);
        let guild2 = GuildId::new(2);

        test_subject.new_server(&guild1).unwrap();
        test_subject.new_server(&guild2).unwrap();
        test_subject.disable_auto_scan(&guild2).unwrap();

        assert_eq!(true, test_subject.is_auto_scan_enabled(&guild1));
        assert_eq!(false, test_subject.is_auto_scan_enabled(&guild2));
        assert_eq!(false, test_subject.is_auto_scan_enabled(&GuildId::new(37)));
    }

    // State machine test
    impl std::fmt::Debug for AppData {
        fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Ok(())
        }
    }

    prop_state_machine! {
      #![proptest_config(Config {
        .. Config::default()
      })]

      #[test]
      fn run_state_machine(
        sequential
        1..30
        =>
        AppData
      );
    }

    pub struct StateMachine;

    #[derive(Clone, Debug)]
    pub enum Transition {
        NewServer(u64),
        UpdateRole((u64, u64)),
        EnableScan(u64),
        DisableScan(u64),
    }

    impl ReferenceStateMachine for StateMachine {
        type State = Vec<i32>;
        type Transition = Transition;

        fn init_state() -> proptest::prelude::BoxedStrategy<Self::State> {
            Just(vec![]).boxed()
        }

        fn transitions(_state: &Self::State) -> BoxedStrategy<Self::Transition> {
            prop_oneof![
              1 => (any::<u64>()).prop_map(Transition::NewServer),
              2 => (any::<u64>(), any::<u64>()).prop_map(Transition::UpdateRole),
              3 => (any::<u64>()).prop_map(Transition::EnableScan),
              4 => (any::<u64>()).prop_map(Transition::DisableScan)
            ]
            .boxed()
        }

        fn apply(state: Self::State, _transition: &Self::Transition) -> Self::State {
            state
        }
    }

    impl StateMachineTest for AppData {
        type Reference = StateMachine;
        type SystemUnderTest = Self;

        fn init_test(
            _ref_state: &<Self::Reference as ReferenceStateMachine>::State,
        ) -> Self::SystemUnderTest {
            AppData::new(":memory:")
        }

        fn apply(
            mut state: Self::SystemUnderTest,
            _ref_state: &<Self::Reference as ReferenceStateMachine>::State,
            transition: <Self::Reference as ReferenceStateMachine>::Transition,
        ) -> Self::SystemUnderTest {
            match transition {
                Transition::NewServer(value) => {
                    let _ = state.new_server(&GuildId::new(value));
                }
                Transition::UpdateRole(values) => {
                    let _ = state.update_server_primary_role(
                        &GuildId::new(values.0),
                        &RoleId::new(values.1),
                    );
                }
                Transition::EnableScan(value) => {
                    let _ = state.enable_auto_scan(&GuildId::new(value));
                }
                Transition::DisableScan(value) => {
                    let _ = state.disable_auto_scan(&GuildId::new(value));
                }
            }

            state
        }
    }
}

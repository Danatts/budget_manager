use crate::budget::list_budgets;
use crate::cli::Command;
use crate::history::list_history;
use crate::record::Record;
use crate::{budget::Budget, history::History};
use rusqlite::{Connection, Result};

pub fn create_budget(conn: &Connection, name: &str, funds: &f64) -> Result<usize> {
    let budget = Budget::new(name, funds);
    budget.insert_budget(conn)
}

pub fn remove_budget(conn: &Connection, id: &u32) -> Result<usize> {
    Budget::delete_budget_by_id(conn, id)
}

pub fn rename_budget(conn: &Connection, id: &u32, name: &str) -> Result<usize> {
    let mut budget = Budget::get_budget_by_id(conn, id)?;
    budget.rename(name);
    budget.update_budget(conn)
}

pub fn increase_funds(conn: &mut Connection, command: &Command) -> Result<usize> {
    if let Command::Increase {
        id,
        amount,
        description,
    } = command
    {
        let mut budget = Budget::get_budget_by_id(conn, id)?;
        let old_value = budget.current_funds;
        budget.increase_funds(amount);
        let new_value = budget.current_funds;
        let record = Record::new(
            *id,
            command.value(),
            *amount,
            old_value,
            new_value,
            description,
        );
        let tx = conn.transaction().unwrap();
        let res = budget.update_budget(&tx)?;
        record.insert_record(&tx)?;
        tx.commit()?;
        Ok(res)
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

pub fn reduce_funds(conn: &mut Connection, command: &Command) -> Result<usize> {
    if let Command::Reduce {
        id,
        amount,
        description,
    } = command
    {
        let mut budget = Budget::get_budget_by_id(conn, id)?;
        let old_value = budget.current_funds;
        budget.reduce_funds(amount);
        let new_value = budget.current_funds;
        let record = Record::new(
            *id,
            command.value(),
            *amount,
            old_value,
            new_value,
            description,
        );
        let tx = conn.transaction().unwrap();
        let res = budget.update_budget(&tx)?;
        record.insert_record(&tx)?;
        tx.commit()?;
        Ok(res)
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

pub fn reset_funds(conn: &mut Connection, command: &Command) -> Result<usize> {
    if let Command::Reset { id, description } = command {
        let mut budget = Budget::get_budget_by_id(conn, id)?;
        let old_value = budget.current_funds;
        budget.reset_funds();
        let new_value = budget.current_funds;
        let record = Record::new(*id, command.value(), 0.0, old_value, new_value, description);
        let tx = conn.transaction().unwrap();
        let res = budget.update_budget(&tx)?;
        record.insert_record(&tx)?;
        tx.commit()?;
        Ok(res)
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

pub fn set_current_funds(conn: &mut Connection, command: &Command) -> Result<usize> {
    if let Command::Current {
        id,
        amount,
        description,
    } = command
    {
        let mut budget = Budget::get_budget_by_id(conn, id)?;
        let old_value = budget.current_funds;
        budget.set_current_funds(amount);
        let new_value = budget.current_funds;
        let record = Record::new(
            *id,
            command.value(),
            *amount,
            old_value,
            new_value,
            description,
        );
        let tx = conn.transaction().unwrap();
        let res = budget.update_budget(&tx)?;
        record.insert_record(&tx)?;
        tx.commit()?;
        Ok(res)
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

pub fn set_initial_funds(conn: &mut Connection, command: &Command) -> Result<usize> {
    if let Command::Initial { id, amount } = command {
        let mut budget = Budget::get_budget_by_id(conn, id)?;
        budget.set_initial_funds(amount);
        let res = budget.update_budget(conn)?;
        Ok(res)
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

pub fn undo_last(conn: &mut Connection) -> Result<usize> {
    let tx = conn.transaction()?;
    let last_record = Record::get_last_record(&tx)?;
    let mut budget = Budget::get_budget_by_id(&tx, &last_record.budget_id)?;
    budget.set_current_funds(&last_record.old_value);
    budget.update_budget(&tx)?;
    let res = Record::delete_record_by_id(&tx, &last_record.record_id.unwrap())?;
    tx.commit()?;
    Ok(res)
}

pub fn print_budgets(conn: &Connection) -> Result<()> {
    let budgets = Budget::get_all_budgets(conn)?;
    Ok(list_budgets(&budgets))
}

pub fn print_history(conn: &Connection, id: &Option<u32>, limit: &Option<u32>) -> Result<()> {
    if let Some(id) = id {
        let list = History::get_history_by_budget_id(conn, *id, limit)?;
        Ok(list_history(&list))
    } else {
        let list = History::get_all_history(conn, limit)?;
        Ok(list_history(&list))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Result<Connection> {
        let mut conn = Connection::open_in_memory().unwrap();
        let tx = conn.transaction()?;
        let query = "
            CREATE TABLE IF NOT EXISTS budgets (
                budget_id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                initial_funds REAL NOT NULL,
                current_funds REAL NOT NULL
            );";
        tx.execute(query, ())?;
        let query = "
            CREATE TABLE IF NOT EXISTS records (
                record_id INTEGER PRIMARY KEY,
                budget_id INTEGER NOT NULL,
                action TEXT NOT NULL,
                amount REAL NOT NULL,
                old_value REAL NOT NULL,
                new_value REAL NOT NULL,
                description TEXT,
                created_at TEXT,
                FOREIGN KEY (budget_id) REFERENCES budgets(budget_id)
            );";
        tx.execute(query, ())?;
        tx.commit()?;
        Ok(conn)
    }

    #[test]
    fn increase_funds_ok() {
        let mut conn = setup_test_db().unwrap();
        let budget = Budget::new("budget_test", &500.0);
        let command = Command::Increase {
            id: 1,
            amount: 100.0,
            description: Some("test_description".to_string()),
        };
        let _ = budget.insert_budget(&conn);
        let _ = increase_funds(&mut conn, &command);
        let budget = Budget::get_budget_by_id(&conn, &1).unwrap();
        assert_eq!(budget.budget_id, Some(1));
        assert_eq!(budget.current_funds, 600.0);
        assert_eq!(budget.initial_funds, 500.0);
    }

    #[test]
    fn reduce_funds_ok() {
        let mut conn = setup_test_db().unwrap();
        let budget = Budget::new("budget_test", &500.0);
        let command = Command::Reduce {
            id: 1,
            amount: 100.0,
            description: Some("test_description".to_string()),
        };
        let _ = budget.insert_budget(&conn);
        let _ = reduce_funds(&mut conn, &command);
        let budget = Budget::get_budget_by_id(&conn, &1).unwrap();
        assert_eq!(budget.budget_id, Some(1));
        assert_eq!(budget.current_funds, 400.0);
        assert_eq!(budget.initial_funds, 500.0);
    }

    #[test]
    fn reset_funds_ok() {
        let mut conn = setup_test_db().unwrap();
        let budget = Budget::new("budget_test", &500.0);
        let command = Command::Reduce {
            id: 1,
            amount: 100.0,
            description: Some("test_description".to_string()),
        };
        let _ = budget.insert_budget(&conn);
        let _ = reduce_funds(&mut conn, &command);
        let budget = Budget::get_budget_by_id(&conn, &1).unwrap();
        assert_eq!(budget.budget_id, Some(1));
        assert_eq!(budget.current_funds, 400.0);
        let command = Command::Reset {
            id: 1,
            description: Some("test_description".to_string()),
        };
        let _ = reset_funds(&mut conn, &command);
        let budget = Budget::get_budget_by_id(&conn, &1).unwrap();
        assert_eq!(budget.budget_id, Some(1));
        assert_eq!(budget.current_funds, 500.0);
    }

    #[test]
    fn current_funds_ok() {
        let mut conn = setup_test_db().unwrap();
        let budget = Budget::new("budget_test", &500.0);
        let command = Command::Current {
            id: 1,
            amount: 100.0,
            description: Some("test_description".to_string()),
        };
        let _ = budget.insert_budget(&conn);
        let _ = set_current_funds(&mut conn, &command);
        let budget = Budget::get_budget_by_id(&conn, &1).unwrap();
        assert_eq!(budget.budget_id, Some(1));
        assert_eq!(budget.current_funds, 100.0);
        assert_eq!(budget.initial_funds, 500.0);
    }

    #[test]
    fn initial_funds_ok() {
        let mut conn = setup_test_db().unwrap();
        let budget = Budget::new("budget_test", &500.0);
        let command = Command::Initial {
            id: 1,
            amount: 100.0
        };
        let _ = budget.insert_budget(&conn);
        let _ = set_initial_funds(&mut conn, &command);
        let budget = Budget::get_budget_by_id(&conn, &1).unwrap();
        assert_eq!(budget.budget_id, Some(1));
        assert_eq!(budget.current_funds, 500.0);
        assert_eq!(budget.initial_funds, 100.0);
    }

    #[test]
    fn undo_last_ok() {
        let mut conn = setup_test_db().unwrap();
        let budget = Budget::new("budget_test", &500.0);
        let command = Command::Reduce {
            id: 1,
            amount: 100.0,
            description: Some("test_description".to_string()),
        };
        let _ = budget.insert_budget(&conn);
        let _ = reduce_funds(&mut conn, &command);
        let _ = reduce_funds(&mut conn, &command);
        let budget = Budget::get_budget_by_id(&conn, &1).unwrap();
        let record = Record::get_last_record(&conn).unwrap();
        assert_eq!(budget.budget_id, Some(1));
        assert_eq!(budget.current_funds, 300.0);
        assert_eq!(record.record_id, Some(2));
        let _ = undo_last(&mut conn);
        let budget = Budget::get_budget_by_id(&conn, &1).unwrap();
        let record = Record::get_last_record(&conn).unwrap();
        assert_eq!(budget.budget_id, Some(1));
        assert_eq!(budget.current_funds, 400.0);
        assert_eq!(record.record_id, Some(1));
    }
}

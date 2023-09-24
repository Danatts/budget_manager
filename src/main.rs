use budget_manager::{
    account::{self, Account},
    cli::{Action::*, CliArgs},
};
use clap::Parser;
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let CliArgs { action } = CliArgs::parse();
    let connection = Connection::open_in_memory()?;

    account::create_accounts_table(&connection).expect("could not create table");

    match action {
        List => account::list_accounts(&connection).expect("could not list accounts"),
        Add {
            amount,
            entity,
            category,
        } => account::add_account(&connection, Account::new(amount, entity, category))
            .expect("could not add new account"),
        Update => unimplemented!(),
        Remove => unimplemented!(),
    }
    account::list_accounts(&connection).expect("could not list accounts");

    Ok(())
}

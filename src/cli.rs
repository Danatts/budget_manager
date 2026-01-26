use crate::budget::create_budget_table;
use crate::record::create_record_table;
use crate::services::{
    create_budget, increase_funds, print_budgets, print_history, reduce_funds, remove_budget,
    rename_budget, reset_funds, set_current_funds, set_initial_funds, undo_last,
};
use clap::{Parser, Subcommand};
use rusqlite::Connection;
use std::path::PathBuf;
use std::process;

#[derive(Subcommand)]
pub enum Command {
    /// Set current budget funds
    Current {
        #[arg(value_name = "ID")]
        id: u32,
        #[arg(value_name = "AMOUNT")]
        amount: f64,
        /// Add small description
        #[arg(long, short, value_name = "DESCRIPTION")]
        description: Option<String>,
    },
    /// Print transaction history
    History {
        #[arg(value_name = "ID")]
        id: Option<u32>,
        #[arg(long, short, value_name = "LIMIT")]
        limit: Option<u32>,
    },
    /// Set initial budget funds
    Initial {
        #[arg(value_name = "ID")]
        id: u32,
        #[arg(value_name = "AMOUNT")]
        amount: f64,
    },
    /// Increase budget funds
    Increase {
        #[arg(value_name = "ID")]
        id: u32,
        #[arg(value_name = "AMOUNT")]
        amount: f64,
        /// Add small description
        #[arg(long, short, value_name = "DESCRIPTION")]
        description: Option<String>,
    },
    /// List all budgets
    List,
    /// Create a new budget
    New {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "FUNDS")]
        funds: f64,
    },
    /// Reduce budget funds
    Reduce {
        #[arg(value_name = "ID")]
        id: u32,
        #[arg(value_name = "AMOUNT")]
        amount: f64,
        /// Add small description
        #[arg(long, short, value_name = "DESCRIPTION")]
        description: Option<String>,
    },
    /// Remove a budget
    Remove {
        #[arg(value_name = "ID")]
        id: u32,
    },
    /// Rename a budget
    Rename {
        #[arg(value_name = "ID")]
        id: u32,
        #[arg(value_name = "NEW NAME")]
        name: String,
    },
    /// Reset a budget to initial funds
    Reset {
        #[arg(value_name = "ID")]
        id: u32,
        /// Add small description
        #[arg(long, short, value_name = "DESCRIPTION")]
        description: Option<String>,
    },
    /// Undo last action
    Undo,
}

impl Command {
    pub fn value(&self) -> &str {
        match self {
            Self::Current {
                id: _,
                amount: _,
                description: _,
            } => "Set current funds",
            Self::Increase {
                id: _,
                amount: _,
                description: _,
            } => "Increase funds",
            Self::Initial { id: _, amount: _ } => "Set initial funds",
            Self::Reduce {
                id: _,
                amount: _,
                description: _,
            } => "Reduce funds",
            Self::Reset {
                id: _,
                description: _,
            } => "Reset funds",
            _ => "",
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    /// Select a database file
    #[arg(long, short, value_name = "FILE NAME")]
    pub database: Option<PathBuf>,
}

pub fn run(conn: &mut Connection, command: Command) {
    if let Err(error) = create_budget_table(&conn) {
        eprintln!("Error: {}.", error);
        process::exit(1);
    }

    if let Err(error) = create_record_table(&conn) {
        eprintln!("Error: {}.", error);
        process::exit(1);
    }

    match &command {
        Command::Current {
            id: _,
            amount: _,
            description: _,
        } => match set_current_funds(conn, &command) {
            Ok(rows) => println!("{} record updates", rows),
            Err(error) => eprintln!("Error: {}", error),
        },
        Command::History { id, limit } => match print_history(&conn, id, limit) {
            Ok(_) => {}
            Err(error) => eprintln!("Error: {}", error),
        },
        Command::Increase {
            id: _,
            amount: _,
            description: _,
        } => match increase_funds(conn, &command) {
            Ok(rows) => println!("{} record updates", rows),
            Err(error) => eprintln!("Error: {}", error),
        },
        Command::Initial { id: _, amount: _ } => match set_initial_funds(conn, &command) {
            Ok(rows) => println!("{} record updates", rows),
            Err(error) => eprintln!("Error: {}", error),
        },
        Command::List => match print_budgets(&conn) {
            Ok(_) => {}
            Err(error) => eprintln!("Error: {}", error),
        },
        Command::New { name, funds } => match create_budget(&conn, name, funds) {
            Ok(rows) => println!("{} record inserted.", rows),
            Err(error) => eprintln!("Error: {}", error),
        },
        Command::Reduce {
            id: _,
            amount: _,
            description: _,
        } => match reduce_funds(conn, &command) {
            Ok(rows) => println!("{} record updates", rows),
            Err(error) => eprintln!("Error: {}", error),
        },
        Command::Remove { id } => match remove_budget(&conn, id) {
            Ok(rows) => println!("{} record deleted.", rows),
            Err(error) => eprintln!("Error: {}", error),
        },
        Command::Rename { id, name } => match rename_budget(&conn, id, name) {
            Ok(rows) => println!("{} record updated.", rows),
            Err(error) => eprintln!("Error: {}", error),
        },
        Command::Reset {
            id: _,
            description: _,
        } => match reset_funds(conn, &command) {
            Ok(rows) => println!("{} record updated.", rows),
            Err(error) => eprintln!("Error: {}", error),
        },
        Command::Undo => match undo_last(conn) {
            Ok(rows) => println!("{} actions undone", rows),
            Err(error) => eprintln!("Error: {}", error),
        },
    }
}

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// List all accounts
    List,
    /// Add a new account
    Add {
        /// Money value
        value: f64,
        /// Name of entity
        entity: String,
        /// Name of category
        category: String,
    },
    /// Update an account
    Update,
    /// Remove an account
    Remove,
}

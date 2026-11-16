mod board;
mod cli;
mod commands;
mod error;
mod git;
mod licenses;
mod managed;
mod manifest;
mod output;
mod project;
mod stacks;
mod template;

use std::process::ExitCode;

use clap::{CommandFactory, Parser};

use crate::cli::{Cli, Command};
use crate::error::Result;
use crate::output::{Output, Reporter};

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.version {
        println!("aboot {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let Some(command) = cli.command else {
        Cli::command().print_help().ok();
        eprintln!();
        return ExitCode::from(2);
    };

    let reporter = Reporter {
        json: cli.json,
        quiet: cli.quiet,
    };

    let result: Result<Output> = match &command {
        Command::New(args) => commands::new::run(args, &reporter),
        Command::Add(args) => commands::add::run(args, &reporter),
        Command::Status(args) => commands::status::run(args, &reporter),
        Command::Board { cmd } => commands::board::run(cmd, &reporter),
        Command::Doctor(args) => commands::doctor::run(args, &reporter),
        Command::Update(args) => commands::update::run(args, &reporter),
        Command::List(args) => commands::list::run(args, &reporter),
    };

    match result {
        Ok(output) => {
            reporter.print(&output);
            ExitCode::from(output.exit_code)
        }
        Err(err) => {
            reporter.print_error(&err);
            ExitCode::from(1)
        }
    }
}

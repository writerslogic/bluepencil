use anyhow::Result;
use clap::CommandFactory;
use clap_complete::Shell;

use crate::cli::Cli;
use crate::exit::Status;

pub fn run(shell: Shell) -> Result<Status> {
    clap_complete::generate(shell, &mut Cli::command(), "bluepencil", &mut std::io::stdout());
    Ok(Status::Ok)
}

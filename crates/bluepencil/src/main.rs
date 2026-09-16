mod cli;
pub mod commands;
mod config;
mod context;
mod exit;
mod glob;
mod output;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    quiet_broken_pipe();
    let cli = cli::Cli::parse();
    match commands::run(cli) {
        Ok(code) => code.into(),
        Err(err) => {
            eprintln!("bluepencil: {err:#}");
            exit::Status::Error.into()
        }
    }
}

/// Exit quietly when output is piped into something like `head` that closes early.
fn quiet_broken_pipe() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info
            .payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| info.payload().downcast_ref::<&str>().copied())
            .unwrap_or_default();
        if msg.contains("Broken pipe") || msg.contains("BrokenPipe") {
            std::process::exit(0);
        }
        default(info);
    }));
}

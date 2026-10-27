mod args;
mod options;
mod output;
mod runtime;

use anyhow::Result;
pub use args::{command, help_options};
use cli_command_kit::Mode;
use std::{ffi::OsString, io::Write};
use tokio_util::sync::CancellationToken;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("operation cancelled")]
    Cancelled,
    #[error("operation exceeded its deadline")]
    Timeout,
}

pub async fn entry(
    args: impl IntoIterator<Item = OsString>,
    output: &mut impl Write,
    diagnostics: &mut impl Write,
    mode: Mode,
    cancel: &CancellationToken,
) -> u8 {
    let mut command =
        cli_command_kit::setup(command(), mode, &help_options()).expect("valid command tree");
    let parsed = command
        .clone()
        .try_get_matches_from(args)
        .and_then(|matches| args::validate(&mut command, matches));
    let result: Result<()> = match parsed {
        Ok(matches) => runtime::perform(&matches, output, diagnostics, mode, cancel).await,
        Err(error) if !error.use_stderr() => output
            .write_all(error.to_string().as_bytes())
            .map_err(Into::into),
        Err(error) => {
            let _ = write!(
                diagnostics,
                "{}",
                cli_command_kit::display_message(&error.to_string())
            );
            return 1;
        }
    };
    match result {
        Ok(()) => 0,
        Err(error) => {
            let _ = writeln!(
                diagnostics,
                "{} {}",
                if mode == Mode::Agent {
                    "ERR:"
                } else {
                    "[ERROR]"
                },
                cli_command_kit::display_message(&format!("{error:#}"))
            );
            if matches!(error.downcast_ref::<ApiError>(), Some(ApiError::Cancelled)) {
                130
            } else {
                1
            }
        }
    }
}

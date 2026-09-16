use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    /// `check` found threshold violations.
    Failed,
    Error,
}

impl From<Status> for ExitCode {
    fn from(s: Status) -> Self {
        match s {
            Status::Ok => ExitCode::SUCCESS,
            Status::Failed => ExitCode::from(1),
            Status::Error => ExitCode::from(2),
        }
    }
}

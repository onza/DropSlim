#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcome {
    Ok,
    Failed,
    Cancelled,
}

pub fn exit_code(outcome: RunOutcome) -> u8 {
    match outcome {
        RunOutcome::Ok => 0,
        RunOutcome::Failed => 1,
        RunOutcome::Cancelled => 130,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_outcomes_to_exit_codes() {
        assert_eq!(exit_code(RunOutcome::Ok), 0);
        assert_eq!(exit_code(RunOutcome::Failed), 1);
        assert_eq!(exit_code(RunOutcome::Cancelled), 130);
    }
}

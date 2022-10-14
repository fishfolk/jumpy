use bevy::ecs::schedule::ShouldRun;

/// Helper trait for dealing with the [`ShouldRun`] type
pub trait ShouldRunExt {
    fn new(should_run: bool, check_again: bool) -> Self;
    fn should_run(&self) -> bool;
    fn check_again(&self) -> bool;
    fn merge(&self, other: Self) -> Self;
}

impl ShouldRunExt for ShouldRun {
    fn new(should_run: bool, check_again: bool) -> Self {
        match (should_run, check_again) {
            (true, true) => ShouldRun::YesAndCheckAgain,
            (true, false) => ShouldRun::Yes,
            (false, true) => ShouldRun::NoAndCheckAgain,
            (false, false) => ShouldRun::No,
        }
    }

    fn should_run(&self) -> bool {
        match self {
            ShouldRun::YesAndCheckAgain | ShouldRun::Yes => true,
            ShouldRun::NoAndCheckAgain | ShouldRun::No => false,
        }
    }

    fn check_again(&self) -> bool {
        match self {
            ShouldRun::Yes | ShouldRun::No => false,
            ShouldRun::YesAndCheckAgain | ShouldRun::NoAndCheckAgain => true,
        }
    }

    fn merge(&self, other: Self) -> Self {
        Self::new(
            self.should_run() || other.should_run(),
            self.check_again() || other.check_again(),
        )
    }
}

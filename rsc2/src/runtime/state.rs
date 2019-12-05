use crate::hook::AgentHook;
use crate::runtime::Commands;
use std::cell::Cell;
use std::io;

macro_rules! bit_flag {
    ($set_f:ident as $val:literal; $check_f:ident) => {
        pub(super) fn $check_f(&self) -> bool {
            let val: u32 = $val;
            self.0.get() & 2u8.pow(val) != 0
        }
        pub(super) fn $set_f(&self) {
            let val: u32 = $val;
            let old = self.0.take();
            self.0.set(old | 2u8.pow(val));
        }
    };
}

#[derive(Default, Debug)]
pub(super) struct StateMachine(Cell<u8>);

impl StateMachine {
    pub(super) fn reset(&self) {
        drop(self.0.take());
    }
    bit_flag!(launched as 0; is_launched);
    bit_flag!(initgame as 1; is_initgame);
    bit_flag!(inreplay as 2; is_inreplay);
    bit_flag!(ingame as 3; is_ingame);
    bit_flag!(ended as 4; is_ended);

    pub(super) fn validate<A: AgentHook>(&self, command: &Commands<A>) -> io::Result<()> {
        let fast_err = |message: &'static str| -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::InvalidInput, message))
        };
        let is_launched = || -> io::Result<()> {
            if self.is_launched() {
                return Ok(());
            }
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Game has not been flaged as launched (if an instance is running send a Comands::Launched to connect)",
            ))
        };
        match command {
            Commands::Launched { .. } => {
                if self.is_launched() {
                    return fast_err("Game is already launched");
                }
            }
            Commands::CreateGame { .. } => {
                is_launched()?;
                if self.is_ingame() || self.is_inreplay() || self.is_initgame() {
                    return fast_err("Cannot create a game while one is running");
                }
            }
            Commands::JoinGame { .. } => {
                is_launched()?;
                if self.is_ingame() || self.is_inreplay() {
                    return fast_err("Cannot join a game while one is running");
                }
            }
            Commands::StartReplay { .. } => {
                is_launched()?;
                if self.is_initgame() || self.is_ingame() {
                    return fast_err("Cannot play a replay while a game is running");
                }
            }
            Commands::RestartGame => {
                is_launched()?;
                if !self.is_ended() || !self.is_ingame() {
                    return fast_err("Cannot restart a game while one is running");
                }
            }
            Commands::LeaveGame => {
                is_launched()?;
                if !self.is_ended() {
                    return fast_err("Cannot leave a game while one is running");
                }
            }
            Commands::QuitGame => {
                is_launched()?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod set {
    use super::StateMachine;

    #[test]
    fn flags() {
        let sm = StateMachine::default();
        sm.launched();
        assert!(sm.is_launched());
        sm.reset();

        sm.initgame();
        assert!(sm.is_initgame());
        sm.reset();

        sm.inreplay();
        assert!(sm.is_inreplay());
        sm.reset();

        sm.ingame();
        assert!(sm.is_ingame());
        sm.reset();

        sm.ended();
        assert!(sm.is_ended());
        sm.reset();

        assert_eq!(sm.0.get(), 0)
    }

    #[test]
    fn multiple_flags() {
        let sm = StateMachine::default();

        sm.launched();
        sm.initgame();
        sm.ingame();
        sm.ended();

        assert!(sm.is_launched());
        assert!(sm.is_initgame());
        assert!(sm.is_ingame());
        assert!(!sm.is_inreplay());
        assert!(sm.is_ended());

        sm.reset();
        assert_eq!(sm.0.get(), 0)
    }
}

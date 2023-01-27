use super::{Command, CommandKind};

pub struct Channel {
    command: String,
    control: [u8; 3]
}

impl Channel {
    pub fn new(control: &[u8; 3], command: &'static str) -> Channel {
        Channel {
            control: *control,
            command: command.to_string()
        }
    }

    pub fn open(&self) -> Vec<u8> {
        Command::preflight(&self.command, &self.control)
    }

    pub fn com(&self, kind: CommandKind) -> Vec<u8> {
        Command::for_kind(kind, &self.control)
    }
}

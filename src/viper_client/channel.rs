use super::{Command, CommandKind};

pub struct Channel {
    command: String,
    control: [u8; 2]
}

impl Channel {
    pub fn new(control: &[u8; 2], command: &'static str) -> Channel {
        Channel {
            control: *control,
            command: command.to_string()
        }
    }

    pub fn open(&self) -> Vec<u8> {
        Command::channel(&self.command, &self.control, None)
    }

    pub fn com(&self, kind: CommandKind) -> Vec<u8> {
        Command::for_kind(kind, &self.control)
    }
}

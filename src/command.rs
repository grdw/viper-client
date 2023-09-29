use serde::{Deserialize, Serialize};

const OPEN:  [u8; 8] = [0xcd, 0xab, 0x01, 0x00, 0x07, 0x00, 0x00, 0x00];
const CLOSE: [u8; 8] = [0xef, 0x01, 0x03, 0x00, 0x02, 0x00, 0x00, 0x00];

pub enum CommandKind {
    UAUT(String),
    UCFG(String),
    RemoveAllUsers(String),
    ActivateUser(String),
    INFO,
    FRCG
}

pub struct Command { }

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Base {
    message: String,
    message_type: String,
    message_id: u8
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct UAUT {
    #[serde(flatten)]
    base: Base,
    user_token: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct UCFG {
    #[serde(flatten)]
    base: Base,
    addressbooks: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RemoveAllUsers {
    #[serde(flatten)]
    base: Base,
    requester: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ActivateUser {
    #[serde(flatten)]
    base: Base,
    email: String,
    description: String,
}

impl Command {
    pub fn for_kind(kind: CommandKind, control: &[u8]) -> Vec<u8> {
        let message_type = String::from("request");

        let json = match kind {
            CommandKind::UAUT(token) => {
                let uaut = UAUT {
                    base: Base {
                        message: String::from("access"),
                        message_id: 1,
                        message_type,
                    },
                    user_token: token
                };

                serde_json::to_string(&uaut).unwrap()
            },

            CommandKind::UCFG(addressbooks) => {
                let ucfg = UCFG {
                    base: Base {
                        message: String::from("get-configuration"),
                        message_id: 2,
                        message_type,
                    },
                    addressbooks
                };

                serde_json::to_string(&ucfg).unwrap()
            },

            CommandKind::INFO => {
                let info = Base {
                    message: String::from("server-info"),
                    message_id: 1,
                    message_type
                };

                serde_json::to_string(&info).unwrap()
            },

            CommandKind::FRCG => {
                let frcg = Base {
                    message: String::from("rcg-get-params"),
                    message_id: 121,
                    message_type
                };

                serde_json::to_string(&frcg).unwrap()
            },

            CommandKind::RemoveAllUsers(requester) => {
                let fact = RemoveAllUsers {
                    base: Base {
                        message: String::from("remove-all-users"),
                        message_id: 1,
                        message_type
                    },
                    requester
                };

                serde_json::to_string(&fact).unwrap()
            },

            CommandKind::ActivateUser(email) => {
                let fact = ActivateUser {
                    email,
                    description: String::from("viper-client"),
                    base: Base {
                        message: String::from("activate-user"),
                        message_id: 1,
                        message_type
                    }
                };

                serde_json::to_string(&fact).unwrap()
            }
        };

        Command::make(&json.as_bytes(), control)
    }

    pub fn buffer_length(b2: u8, b3: u8) -> usize {
        u16::from_le_bytes([b2, b3]) as usize
    }

    pub fn channel(command: &String,
                   control: &[u8],
                   extra: Option<&[u8]>) -> Vec<u8> {

        let com_b = command.as_bytes();

        let tail = match extra {
            Some(bytes) => {
                let len = (bytes.len() + 1) as u8;
                let start = [0, 0, len, 0, 0, 0];
                [&start[..], &bytes[..], &[0]].concat()
            },
            None => vec![0]
        };

        let total = [
            &OPEN,
            &com_b[..],
            &control[..],
            &tail[..]
        ].concat();

        let header = Command::header(&total);

        [&header, &total[..]].concat()
    }

    pub fn close(channel: &[u8]) -> Vec<u8> {
        let total = [
            &CLOSE,
            &channel[..],
        ].concat();

        let header = Command::header(&total);

        [&header, &total[..]].concat()
    }

    pub fn make(b_com: &[u8], control: &[u8]) -> Vec<u8> {
        let mut header = Command::header(b_com);
        header[4] = control[0];
        header[5] = control[1];

        [&header, &b_com[..]].concat()
    }

    fn header(total: &[u8]) -> [u8; 8] {
        let len = total.len().to_le_bytes();

        [0x00, 0x06, len[0], len[1], 0x00, 0x00, 0x00, 0x00]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_kind() {
        let control = [1, 2];

        let channel = Command::for_kind(CommandKind::UAUT("token".to_string()), &control);
        assert_eq!(channel.len(), 89);
    }

    #[test]
    fn test_content_length() {
        let control = [1, 2];
        let list = vec![
            (94, 94, 0),
            (117, 117, 0),
            (367, 111, 1),
            (752, 240, 2),
            (951, 183, 3)
        ];

        for (byte_length, b2, b3) in list {
            let mut s = String::from("A");
            s = s.repeat(byte_length);
            let b = Command::make(s.as_bytes(), &control);
            assert_eq!(b[2], b2);
            assert_eq!(b[3], b3);
        }
    }

    #[test]
    fn test_command_channel() {
        let control = [1, 2];
        let command = String::from("UCFG");
        let b = Command::channel(&command, &control, None);
        assert_eq!(b[2], 15);
        assert_eq!(b[3], 0);

        let b = Command::channel(
            &command,
            &control,
            Some(&[10, 10, 10])
        );
        assert_eq!(b[2], 24);
        assert_eq!(b[3], 0);
    }

    #[test]
    fn test_command_close() {
        let control = [1, 2];
        let b = Command::close(&control);

        assert_eq!(b[2], 10);
        assert_eq!(b[3], 0);
        assert_eq!(&b[16..18], &control);
    }

    #[test]
    fn test_buffer_length() {
        assert_eq!(Command::buffer_length(94, 0), 94);
        assert_eq!(Command::buffer_length(109, 0), 109);
        assert_eq!(Command::buffer_length(103, 1), 359);
        assert_eq!(Command::buffer_length(232, 2), 744);
        assert_eq!(Command::buffer_length(175, 3), 943);
    }
}

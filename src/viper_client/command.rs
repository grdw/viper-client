// This is the command prefix I see flying by
// every time
const COMMAND_HEADER: [u8; 8] = [205, 171, 1,  0, 7, 0, 0, 0];
// This is another header that's pretty consistent,
// not sure what it's for tbh:
const UNKNOWN_HEADER: [u8; 8] = [239, 1,   3,  0, 2, 0, 0, 0];

pub struct Command { }

impl Command {
    pub fn preflight(command: &'static str, control: &[u8]) -> Vec<u8> {
        Command::cmd(command, &[], control)
    }

    pub fn buffer_length(b2: u8, b3: u8) -> usize {
        let b2 = b2 as usize;
        let b3 = b3 as usize;

        (b3 * 255) + b2 + b3
    }

    pub fn cmd(command: &'static str,
           extra: &[u8],
           control: &[u8]) -> Vec<u8> {

        let com_b = command.as_bytes();

        let total = [
            &COMMAND_HEADER,
            &com_b[..],
            &control[..],
            &extra[..]
        ].concat();

        let header = Command::header(&total);

        [&header, &total[..]].concat()
    }

    pub fn release(control: &[u8]) -> Vec<u8> {
        let total = [&UNKNOWN_HEADER, &control[0..2]].concat();

        let header = Command::header(&total);

        [&header, &total[..]].concat()
    }

    pub fn make(b_com: &[u8], control: &[u8]) -> Vec<u8> {
        let mut header = Command::header(b_com);
        header[4] = control[0];
        header[5] = control[1];
        header[6] = control[2];

        [&header, &b_com[..]].concat()
    }

    fn header(total: &[u8]) -> [u8; 8] {
        let (length, second) = Command::byte_lengths(&total);

        [0, 6, length, second, 0, 0, 0, 0]
    }

    fn byte_lengths(bytes: &[u8]) -> (u8, u8) {
        let second = bytes.len() / 255;
        let length = (bytes.len() % 255) - second;

        (length as u8, second as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_length() {
        let control = [1, 2, 0];
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
    fn test_command_make() {
        let control = [1, 2, 0];
        let b = Command::cmd("UCFG", &[], &control);
        assert_eq!(b[2], 15);
        assert_eq!(b[3], 0);

        let b = Command::cmd("UCFG", &[10, 10, 10], &control);
        assert_eq!(b[2], 18);
        assert_eq!(b[3], 0);
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

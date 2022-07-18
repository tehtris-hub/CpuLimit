//! An iterator over the fields of `/proc/<pid>/stat` files.
//!
//! The second field of stat files (`comm`) is an arbitrary string
//! that might contain whitespace, making the straightforward
//! [`str::split_whitespace`] parsing impossible.
//!
//! See `man proc` for a list of the fields in the file.

use std::fs;
use std::io;

use crate::pid::Pid;

/// The content of a `/proc/<pid>/stat` file.
pub struct StatFile(String);

/// An iterator over the fields of a [`StatFile`].
pub struct StatFileIter<'s> {
    data: &'s str,
    idx: usize,
    state: State,
}

/// The state of a `StatFileIter`.
#[derive(PartialEq)]
enum State {
    /// Just instantiated, the next field is the first (PID).
    Pid,
    /// The next field to yield is the command name.
    Command,
    /// The remaining fields are simply separated by whitespace.
    Normal,
}

impl StatFile {
    /// Opens the `/proc/<pid>/stat` file.
    pub fn open(pid: Pid) -> io::Result<Self> {
        let stat = fs::read_to_string(format!("/proc/{pid}/stat"))?;
        Ok(Self(stat))
    }

    /// Creates an iterator over the fields of the file.
    pub fn iter(&self) -> StatFileIter<'_> {
        self.0[..].into()
    }
}

impl<'a> From<&'a str> for StatFileIter<'a> {
    fn from(data: &'a str) -> Self {
        Self {
            data,
            idx: 0,
            state: State::Pid,
        }
    }
}

impl<'a> Iterator for StatFileIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let State::Command = self.state {
            // find the last parenthesis as it marks the end of the command name
            let idx = self.data.rfind(')')?;
            self.idx += 1; // skip first parenthesis
            let res = &self.data[self.idx..idx];

            // the following fields are all whitespace-separated
            self.state = State::Normal;
            self.idx = idx + 2; // place idx on the next field

            Some(res)
        } else {
            if self.state == State::Pid {
                self.state = State::Command;
            }

            // yield the next whitespace-separated field
            let idx = self.idx + self.data[self.idx..].find(char::is_whitespace)?;
            let res = &self.data[self.idx..idx];
            self.idx = idx + 1;
            Some(res)
        }
    }
}

#[cfg(test)]
mod test {
    use super::StatFile;
    use super::StatFileIter;

    #[test]
    fn standard_stat() {
        let stat = "128377 (cat) R 127912 128377 127912 34817 128377 4194304 90 0 0 0 0 0 0 0 25 5 1 0 7545849 18751488 252 18446744073709551615 94742542643200 94742542658614 140726597052192 0 0 0 0 0 0 0 0 0 17 0 0 0 0 0 0 94742542670560 94742542671976 94742570721280 140726597055035 140726597055055 140726597055055 140726597058539 0\n";
        let mut stat = StatFileIter::from(stat);

        assert_eq!(stat.next(), Some("128377"));
        assert_eq!(stat.next(), Some("cat"));
        assert_eq!(stat.next(), Some("R"));
        assert_eq!(stat.next(), Some("127912"));
        assert_eq!(stat.nth(52 - 4 - 1), Some("0"));
    }

    #[test]
    fn evil_program_name() {
        let stat = "144650 (evil program x) name!) S 120869 144650 120869 34819 144650 4194304 94 0 0 0 0 0 0 0 15 -5 1 0 8684651 18751488 274 18446744073709551615 94787199291392 94787199306806 140721558631744 0 0 0 0 0 0 0 0 0 17 3 0 0 0 0 0 94787199318752 94787199320168 94787216977920 140721558639669 140721558639689 140721558639689 140721558642667 42\n";
        let mut stat = StatFileIter::from(stat);

        assert_eq!(stat.next(), Some("144650"));
        assert_eq!(stat.next(), Some("evil program x) name!"));
        assert_eq!(stat.next(), Some("S"));
        assert_eq!(stat.next(), Some("120869"));
        assert_eq!(stat.nth(52 - 4 - 1), Some("42"));
    }

    #[test]
    fn parse_real_file() {
        let pid = std::process::id();
        let stat = StatFile::open(pid.into()).unwrap();
        let mut stat = stat.iter();
        assert_eq!(stat.next(), Some(&pid.to_string()[..]));
    }
}

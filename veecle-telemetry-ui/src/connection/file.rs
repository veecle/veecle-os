//! See [FileConnection].

use std::io::BufRead;
use std::{fs, io};

use anyhow::Context;

use crate::connection::{Connection, ConnectionMessage};

/// A connection to a file on the local filesystem.
#[derive(Debug)]
pub struct FileConnection {
    /// File path.
    path: String,

    reader: io::Lines<io::BufReader<fs::File>>,

    done: bool,
}

impl FileConnection {
    /// Create a new connection to a local path.
    pub fn new_boxed(path: String) -> anyhow::Result<Box<dyn Connection>> {
        let file = fs::File::open(&path).with_context(|| format!("opening file {path}"))?;
        let reader = io::BufReader::new(file);
        let reader = reader.lines();

        Ok(Box::new(Self {
            path,
            reader,
            done: false,
        }))
    }
}

impl Connection for FileConnection {
    fn try_recv(&mut self) -> Option<ConnectionMessage> {
        match self.reader.next() {
            Some(Ok(line)) => Some(ConnectionMessage::Line(line)),
            Some(Err(error)) => Some(ConnectionMessage::Error(
                anyhow::anyhow!(error).context("file connection error"),
            )),
            None => {
                self.done = true;
                None
            }
        }
    }

    fn is_continuous(&self) -> bool {
        false
    }

    fn is_done(&self) -> bool {
        self.done
    }
}

impl std::fmt::Display for FileConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "file ({})", self.path)
    }
}

//! See [FileContentsConnection].

use std::io::{BufRead, Cursor};
use std::sync::Arc;

use crate::connection::{Connection, ConnectionMessage};

/// The contents of a file that was dropped onto the app.
#[derive(Clone)]
pub struct FileContents {
    /// File name.
    pub name: String,
    /// File content in bytes.
    pub bytes: Arc<[u8]>,
}

impl std::fmt::Debug for FileContents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileContents")
            .field("name", &self.name)
            .field("bytes", &format_args!("{} bytes", self.bytes.len()))
            .finish()
    }
}

/// A connection to an in memory file in the browser.
#[derive(Debug)]
pub struct FileContentsConnection {
    file_name: String,
    bytes: Cursor<Arc<[u8]>>,

    done: bool,
}

impl FileContentsConnection {
    /// Create a new connection to a file in memory.
    pub fn new_boxed(file_contents: FileContents) -> Box<dyn Connection> {
        Box::new(Self {
            file_name: file_contents.name.clone(),
            bytes: Cursor::new(file_contents.bytes),

            done: false,
        })
    }
}

impl Connection for FileContentsConnection {
    fn try_recv(&mut self) -> Option<ConnectionMessage> {
        let mut buffer = String::new();

        match self.bytes.read_line(&mut buffer) {
            Ok(0) => {
                self.done = true;
                None
            }
            Ok(_) => Some(ConnectionMessage::Line(buffer)),
            Err(error) => Some(ConnectionMessage::Error(
                anyhow::anyhow!(error).context("file contents connection error"),
            )),
        }
    }

    fn is_continuous(&self) -> bool {
        false
    }

    fn is_done(&self) -> bool {
        self.done
    }
}

impl std::fmt::Display for FileContentsConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "file ({})", self.file_name)
    }
}

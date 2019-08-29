use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::{RawFd, FromRawFd};

/// Two pipes are needed for each VmApp
/// One pipe used to send requests into VM
/// The other pipe used to read responses from VM
#[derive(Debug)]
pub struct PipePair {
    pub response_reader: File,
    pub requests_input: File,
}

impl PipePair {
    pub fn new(write_fd: RawFd, read_fd: RawFd) -> Self {
        PipePair{
            requests_input: unsafe { File::from_raw_fd(write_fd) },
            response_reader: unsafe { File::from_raw_fd(read_fd) },
        }
    }

    pub fn try_clone(&self) -> io::Result<PipePair> {
        Ok(PipePair{
            requests_input: self.requests_input.try_clone()?,
            response_reader: self.response_reader.try_clone()?,
        })
    }
}

impl Read for PipePair {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.response_reader.read(buf)
    }
}

impl Write for PipePair {
    fn flush(&mut self) -> io::Result<()> {
        self.requests_input.flush()
    }

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.requests_input.write(buf)
    }
}

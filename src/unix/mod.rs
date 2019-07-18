use std::io;
use std::os::unix::io::RawFd;

use crate::poll::convert_ready_to_interests;
use crate::{Evented, Poll, PollOpt, Ready, Token};

use mio::event::Source;
use mio::unix::SourceFd;

#[derive(Debug)]
pub struct EventedFd<'a>(pub &'a RawFd);

impl<'a> Evented for EventedFd<'a> {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        _opts: PollOpt,
    ) -> io::Result<()> {
        let registry = unsafe { poll.registry() };

        SourceFd(&self.0).register(
            registry,
            mio::Token(token.0),
            convert_ready_to_interests(interest).unwrap(),
        )
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        _opts: PollOpt,
    ) -> io::Result<()> {
        let registry = unsafe { poll.registry() };
        SourceFd(&self.0).reregister(
            registry,
            mio::Token(token.0),
            convert_ready_to_interests(interest).unwrap(),
        )
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        let registry = unsafe { poll.registry() };
        SourceFd(&self.0).deregister(registry)
    }
}

pub use mio_old::unix::UnixReady;

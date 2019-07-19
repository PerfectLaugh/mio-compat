use std::io;

use crate::poll::convert_interests_to_ready;
use crate::{Poll, PollOpt, Ready, Token};

pub trait Evented {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
        -> io::Result<()>;
    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()>;
    fn deregister(&self, poll: &Poll) -> io::Result<()>;
}

pub struct EventedSource<'a, 'b: 'a, E: Evented + ?Sized>(&'a E, &'b Poll);

impl<'a, 'b: 'a, E: Evented + ?Sized> EventedSource<'a, 'b, E> {
    pub fn new(ev: &'a E, poll: &'b Poll) -> EventedSource<'a, 'b, E>
    where
        E: Evented,
    {
        EventedSource(ev, poll)
    }
}

impl<'a, 'b: 'a, E: Evented + ?Sized> mio::event::Source for EventedSource<'a, 'b, E> {
    fn register(
        &self,
        _registry: &mio::Registry,
        token: mio::Token,
        interests: mio::Interests,
    ) -> io::Result<()> {
        self.0.register(
            self.1,
            Token(token.0),
            convert_interests_to_ready(interests),
            PollOpt::edge(),
        )
    }
    fn reregister(
        &self,
        _registry: &mio::Registry,
        token: mio::Token,
        interests: mio::Interests,
    ) -> io::Result<()> {
        self.0.reregister(
            self.1,
            Token(token.0),
            convert_interests_to_ready(interests),
            PollOpt::edge(),
        )
    }
    fn deregister(&self, _registry: &mio::Registry) -> io::Result<()> {
        self.0.deregister(self.1)
    }
}

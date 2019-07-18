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

pub struct EventedSource<'a, E: Evented + ?Sized>(&'a E);

impl<'a, E: Evented + ?Sized> EventedSource<'a, E> {
    pub fn new(ev: &'a E) -> EventedSource<'a, E>
    where
        E: Evented,
    {
        EventedSource(ev)
    }
}

impl<'a, E: Evented + ?Sized> mio::event::Source for EventedSource<'a, E> {
    fn register(
        &self,
        registry: &mio::Registry,
        token: mio::Token,
        interests: mio::Interests,
    ) -> io::Result<()> {
        let poll = Poll::from_registry(registry);
        poll.register(
            self.0,
            Token(token.0),
            convert_interests_to_ready(interests),
            PollOpt::edge(),
        )
    }
    fn reregister(
        &self,
        registry: &mio::Registry,
        token: mio::Token,
        interests: mio::Interests,
    ) -> io::Result<()> {
        let poll = Poll::from_registry(registry);
        poll.reregister(
            self.0,
            Token(token.0),
            convert_interests_to_ready(interests),
            PollOpt::edge(),
        )
    }
    fn deregister(&self, registry: &mio::Registry) -> io::Result<()> {
        let poll = Poll::from_registry(registry);
        poll.deregister(self.0)
    }
}

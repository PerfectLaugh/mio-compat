use std::fmt;
use std::io;
use std::sync::RwLock;
use std::time::Duration;

use crate::evented::EventedSource;
use crate::{Event, Events, PollOpt, Ready, Token};

pub struct Poll {
    poll: RwLock<mio::Poll>,
}

impl Poll {
    pub fn new() -> io::Result<Poll> {
        Ok(Poll {
            poll: RwLock::new(mio::Poll::new()?),
        })
    }

    pub fn register<E: ?Sized>(
        &self,
        handle: &E,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()>
    where
        E: crate::Evented,
    {
        validate_args(opts)?;
        let interests = match convert_ready_to_interests(interest) {
            Some(interests) => interests,
            None => return self.deregister(handle),
        };
        self.poll.read().unwrap().registry().register(
            &EventedSource::new(handle, &self),
            mio::Token(token.0),
            interests,
        )
    }

    pub fn reregister<E: ?Sized>(
        &self,
        handle: &E,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()>
    where
        E: crate::Evented,
    {
        validate_args(opts)?;
        let interests = match convert_ready_to_interests(interest) {
            Some(interests) => interests,
            None => return self.deregister(handle),
        };
        self.poll.read().unwrap().registry().reregister(
            &EventedSource::new(handle, &self),
            mio::Token(token.0),
            interests,
        )
    }
    pub fn deregister<E: ?Sized>(&self, handle: &E) -> io::Result<()>
    where
        E: crate::Evented,
    {
        self.poll
            .read()
            .unwrap()
            .registry()
            .deregister(&EventedSource::new(handle, &self))
    }

    pub fn poll(&self, events: &mut Events, timeout: Option<Duration>) -> io::Result<usize> {
        events.clear();
        let mut new_events = mio::Events::with_capacity(events.capacity());
        let size = self.poll.write().unwrap().poll(&mut new_events, timeout)?;
        for event in &new_events {
            events.inner.push(Event::new(
                convert_event_to_ready(event),
                Token(event.token().0),
            ));
        }
        Ok(size)
    }

    pub fn poll_interruptible(
        &self,
        events: &mut Events,
        timeout: Option<Duration>,
    ) -> io::Result<usize> {
        events.clear();
        let mut new_events = mio::Events::with_capacity(events.capacity());
        let size = self
            .poll
            .write()
            .unwrap()
            .poll_interruptible(&mut new_events, timeout)?;
        for event in &new_events {
            events.inner.push(Event::new(
                convert_event_to_ready(event),
                Token(event.token().0),
            ));
        }
        Ok(size)
    }

    pub(crate) unsafe fn registry(&self) -> &mio::Registry {
        &*(self.poll.read().unwrap().registry() as *const mio::Registry)
    }
}

fn validate_args(opts: PollOpt) -> io::Result<()> {
    if !opts.is_edge() || opts.is_oneshot() {
        return Err(io::Error::new(io::ErrorKind::Other, "invalid PollOpt"));
    }

    Ok(())
}

pub(crate) fn convert_ready_to_interests(ready: Ready) -> Option<mio::Interests> {
    use mio::Interests;

    if ready.is_readable() && ready.is_writable() {
        Some(Interests::READABLE | Interests::WRITABLE)
    } else if ready.is_readable() {
        Some(Interests::READABLE)
    } else if ready.is_writable() {
        Some(Interests::WRITABLE)
    } else {
        None
    }
}

pub(crate) fn convert_interests_to_ready(interests: mio::Interests) -> Ready {
    let mut ready = Ready::empty();

    if interests.is_readable() {
        ready |= Ready::readable();
    }
    if interests.is_writable() {
        ready |= Ready::writable();
    }

    ready
}

pub(crate) fn convert_event_to_ready(event: &mio::event::Event) -> Ready {
    let mut ready = Ready::empty();

    if event.is_readable() {
        ready |= Ready::readable();
    }
    if event.is_writable() {
        ready |= Ready::writable();
    }

    ready
}

impl fmt::Debug for Poll {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Poll").finish()
    }
}

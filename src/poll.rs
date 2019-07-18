use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::ops::Fn;
use std::sync::Mutex;
use std::time::Duration;

use crate::evented::EventedSource;
use crate::{Event, Events, PollOpt, Ready, Token};

use lazy_static::lazy_static;

lazy_static! {
    static ref POLL_MAP: Mutex<HashMap<usize, usize>> = Mutex::new(HashMap::new());
}

enum PollInternal {
    Poll(RefCell<mio::Poll>),
    Registry(*const mio::Registry),
}

pub struct Poll(PollInternal);

impl Poll {
    pub fn new() -> io::Result<Poll> {
        Ok(Poll(PollInternal::Poll(RefCell::new(mio::Poll::new()?))))
    }

    pub(crate) fn from_registry(registry: &mio::Registry) -> Poll {
        Poll(PollInternal::Registry(registry))
    }

    pub fn register<S: ?Sized>(
        &self,
        handle: &S,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()>
    where
        S: crate::Evented,
    {
        validate_args(opts)?;
        let interests = match convert_ready_to_interests(interest) {
            Some(interests) => interests,
            None => return Err(io::Error::from(io::ErrorKind::InvalidInput)),
        };
        match &self.0 {
            PollInternal::Poll(internal) => internal.borrow().registry().register(
                &EventedSource::new(handle),
                mio::Token(token.0),
                interests,
            ),
            PollInternal::Registry(registry) => unsafe {
                (**registry).register(&EventedSource::new(handle), mio::Token(token.0), interests)
            },
        }
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
            None => return Err(io::Error::from(io::ErrorKind::InvalidInput)),
        };
        match &self.0 {
            PollInternal::Poll(internal) => internal.borrow().registry().reregister(
                &EventedSource::new(handle),
                mio::Token(token.0),
                interests,
            ),
            PollInternal::Registry(registry) => unsafe {
                (**registry).reregister(&EventedSource::new(handle), mio::Token(token.0), interests)
            },
        }
    }
    pub fn deregister<E: ?Sized>(&self, handle: &E) -> io::Result<()>
    where
        E: crate::Evented,
    {
        match &self.0 {
            PollInternal::Poll(internal) => internal
                .borrow()
                .registry()
                .deregister(&EventedSource::new(handle)),
            PollInternal::Registry(registry) => unsafe {
                (**registry).deregister(&EventedSource::new(handle))
            },
        }
    }

    pub fn poll(&self, events: &mut Events, timeout: Option<Duration>) -> io::Result<usize> {
        events.clear();
        let inner = match &self.0 {
            PollInternal::Poll(poll) => poll,
            _ => return Err(io::Error::from(io::ErrorKind::InvalidData)),
        };
        let mut new_events = mio::Events::with_capacity(events.capacity());
        let size = inner.borrow_mut().poll(&mut new_events, timeout)?;
        for event in &new_events {
            events.push(Event::new(
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
        let inner = match &self.0 {
            PollInternal::Poll(poll) => poll,
            _ => return Err(io::Error::from(io::ErrorKind::InvalidData)),
        };
        let mut new_events = mio::Events::with_capacity(events.capacity());
        let size = inner
            .borrow_mut()
            .poll_interruptible(&mut new_events, timeout)?;
        for event in &new_events {
            events.push(Event::new(
                convert_event_to_ready(event),
                Token(event.token().0),
            ));
        }
        Ok(size)
    }

    pub(crate) fn use_registry<F: Fn(&mio::Registry) -> io::Result<()>>(
        &self,
        func: F,
    ) -> io::Result<()> {
        match &self.0 {
            PollInternal::Poll(internal) => func(internal.borrow().registry()),
            PollInternal::Registry(registry) => unsafe { func(&**registry) },
        }
    }
}

fn validate_args(opts: PollOpt) -> io::Result<()> {
    if !opts.is_edge() {
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
        match self.0 {
            PollInternal::Poll(_) => write!(fmt, "Poll::Poll"),
            PollInternal::Registry(_) => write!(fmt, "Poll::Registry"),
        }
    }
}

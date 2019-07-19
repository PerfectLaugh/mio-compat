use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use crate::{Evented, Poll, PollOpt, Ready, Token};

#[derive(Clone)]
pub struct Registration(Arc<RegistrationInner>);

impl Registration {
    pub fn new2() -> (Registration, SetReadiness) {
        let inner = Arc::new(RegistrationInner::new());
        (Registration(inner.clone()), SetReadiness::new(inner))
    }
}

#[derive(Clone)]
pub struct SetReadiness {
    inner: Arc<RegistrationInner>,
}

impl SetReadiness {
    fn new(inner: Arc<RegistrationInner>) -> SetReadiness {
        SetReadiness { inner }
    }
    pub fn readiness(&self) -> Ready {
        self.inner.readiness()
    }
    pub fn set_readiness(&self, ready: Ready) -> io::Result<()> {
        self.inner.set_readiness(ready)
    }
}

pub struct RegistrationInner {
    waker: RwLock<Option<mio::Waker>>,
    cur_ready: Arc<AtomicUsize>,
}

impl RegistrationInner {
    fn new() -> RegistrationInner {
        RegistrationInner {
            waker: RwLock::new(None),
            cur_ready: Arc::new(AtomicUsize::new(Ready::empty().as_usize())),
        }
    }

    fn readiness(&self) -> Ready {
        Ready::from_usize(self.cur_ready.load(Ordering::Acquire))
    }

    fn set_readiness(&self, ready: Ready) -> io::Result<()> {
        if !ready.is_readable() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        self.cur_ready.store(ready.as_usize(), Ordering::Release);
        self.send_ready(ready)
    }

    fn send_ready(&self, ready: Ready) -> io::Result<()> {
        let waker = self.waker.read().unwrap();
        if waker.is_none() {
            return Ok(());
        }
        if ready.is_readable() {
            waker.as_ref().unwrap().wake()?;
        }
        Ok(())
    }

    fn set_waker(&self, waker: Option<mio::Waker>) {
        let mut waker_store = self.waker.write().unwrap();
        *waker_store = waker;
    }
}

impl Evented for Registration {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        _opts: PollOpt,
    ) -> io::Result<()> {
        if !interest.is_readable() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        let registry = unsafe { poll.registry() };
        self.0
            .set_waker(Some(mio::Waker::new(registry, mio::Token(token.0))?));
        // There are maybe readable registration before register call. We need to wake this up if readable.
        if self.0.readiness().is_readable() {
            // Ignore the result
            drop(self.0.set_readiness(Ready::readable()));
        }
        Ok(())
    }
    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        _opts: PollOpt,
    ) -> io::Result<()> {
        if !interest.is_readable() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        let registry = unsafe { poll.registry() };
        self.0
            .set_waker(Some(mio::Waker::new(registry, mio::Token(token.0))?));
        Ok(())
    }
    fn deregister(&self, _poll: &Poll) -> io::Result<()> {
        self.0.set_waker(None);
        Ok(())
    }
}

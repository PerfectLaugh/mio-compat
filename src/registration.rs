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
    cur_ready: Arc<AtomicUsize>,
}

impl SetReadiness {
    fn new(inner: Arc<RegistrationInner>) -> SetReadiness {
        SetReadiness {
            inner,
            cur_ready: Arc::new(AtomicUsize::new(Ready::empty().as_usize())),
        }
    }
    pub fn readiness(&self) -> Ready {
        Ready::from_usize(self.cur_ready.load(Ordering::Acquire))
    }
    pub fn set_readiness(&self, ready: Ready) -> io::Result<()> {
        if !ready.is_readable() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        self.cur_ready.store(ready.as_usize(), Ordering::Release);
        self.inner.send_ready(ready)
    }
}

pub struct RegistrationInner {
    waker: RwLock<Option<mio::Waker>>,
}

impl RegistrationInner {
    fn new() -> RegistrationInner {
        RegistrationInner {
            waker: RwLock::new(None),
        }
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

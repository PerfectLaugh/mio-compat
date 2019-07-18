use std::io;
use std::sync::{Arc, Mutex};

use crate::{Evented, Poll, PollOpt, Ready, Token};

pub struct Registration(Arc<Mutex<RegistrationInner>>);

impl Registration {
    pub fn new2() -> (Registration, SetReadiness) {
        let inner = Arc::new(Mutex::new(RegistrationInner::new()));
        (Registration(inner.clone()), SetReadiness::new(inner))
    }
}

pub struct SetReadiness {
    inner: Arc<Mutex<RegistrationInner>>,
    cur_ready: Arc<Mutex<Ready>>,
}

impl SetReadiness {
    fn new(inner: Arc<Mutex<RegistrationInner>>) -> SetReadiness {
        SetReadiness {
            inner,
            cur_ready: Arc::new(Mutex::new(Ready::empty())),
        }
    }
    pub fn readiness(&self) -> Ready {
        *self.cur_ready.lock().unwrap()
    }
    pub fn set_readiness(&self, ready: Ready) -> io::Result<()> {
        *self.cur_ready.lock().unwrap() = ready;
        self.inner.lock().unwrap().send_ready(ready)
    }
}

pub struct RegistrationInner {
    waker: Option<mio::Waker>,
    interest: Ready,
}

impl RegistrationInner {
    fn new() -> RegistrationInner {
        RegistrationInner {
            waker: None,
            interest: Ready::empty(),
        }
    }

    fn send_ready(&self, ready: Ready) -> io::Result<()> {
        if self.waker.is_none() {
            return Ok(());
        }
        if !(self.interest & ready).is_empty() {
            self.waker.as_ref().unwrap().wake()?;
        }
        Ok(())
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
        let mut inner = self.0.lock().unwrap();
        let registry = unsafe { poll.registry() };
        inner.waker = Some(mio::Waker::new(registry, mio::Token(token.0))?);
        inner.interest = interest;
        Ok(())
    }
    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        _opts: PollOpt,
    ) -> io::Result<()> {
        let mut inner = self.0.lock().unwrap();
        let registry = unsafe { poll.registry() };
        inner.waker = Some(mio::Waker::new(registry, mio::Token(token.0))?);
        inner.interest = interest;
        Ok(())
    }
    fn deregister(&self, _poll: &Poll) -> io::Result<()> {
        let mut inner = self.0.lock().unwrap();
        inner.waker = None;
        inner.interest = Ready::empty();
        Ok(())
    }
}

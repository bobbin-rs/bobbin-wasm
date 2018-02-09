#![allow(unused_variables)]
use {Error};
use event::Event;

pub type DelegateResult = Result<(), Error>;

pub trait Delegate {
    fn dispatch(&mut self, evt: Event) -> DelegateResult { Ok(())}
}
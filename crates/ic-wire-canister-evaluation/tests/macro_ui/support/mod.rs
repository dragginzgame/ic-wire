use ic_wire::{Limits, Method};
pub struct Read;
impl Method for Read {
    type Request = u64;
    type Response = u64;
    const ID: u64 = 7;
    const NAME: &'static str = "handler";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 128,
        allocation: 512,
        nodes: 16,
        depth: 8,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
pub fn guard(_: &u64) -> Result<(), &'static str> {
    Ok(())
}
// Compile-only surface. Installed ordering and rollback belong to the macro fixture.
pub mod transport {
    pub fn startup() -> Result<(), &'static str> {
        Ok(())
    }
    pub fn argument_size() -> usize {
        0
    }
    pub fn authorize_caller() -> Result<(), &'static str> {
        Ok(())
    }
    pub fn arguments() -> Vec<u8> {
        vec![]
    }
    pub fn trap(message: &str) -> ! {
        panic!("{message}")
    }
    pub fn reply(_: &[u8]) {}
    pub fn reject(_: &str) {}
}

//! ffi/pony.rs - Pony Actor System
use super::error::Result;
use std::os::raw::c_int;

extern "C" {
    fn pony_init_actors() -> c_int;
    fn pony_spawn_search_actor() -> c_int;
}

pub fn init() -> Result<()> {
    unsafe {
        pony_init_actors();
    }
    Ok(())
}

pub fn shutdown() {}

pub fn spawn_actor() -> Result<bool> {
    unsafe {
        let res = pony_spawn_search_actor();
        Ok(res == 0)
    }
}

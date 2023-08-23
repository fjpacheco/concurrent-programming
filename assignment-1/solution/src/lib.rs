pub mod cafeteria;
pub mod conteiners;
pub mod conteiners_states;
pub mod dispenser;
pub mod enums;
pub mod error_dispenser;
pub mod file_orders;
pub mod order;
pub mod periodic_alert;
pub mod set_conteiners;
pub mod traits;
pub mod utils;

pub mod sync {
    use std::time::Duration;

    pub(crate) use std::sync::{atomic::AtomicI64, Arc, Condvar, Mutex, MutexGuard};

    pub(crate) use std::thread;

    pub(crate) fn sleep(_d: Duration) {
        #[cfg(test)]
        let sleep_fn = std::thread::yield_now;

        #[cfg(test)]
        sleep_fn();

        #[cfg(not(test))]
        let sleep_fn = std::thread::sleep;

        #[cfg(not(test))]
        sleep_fn(_d);
    }
}

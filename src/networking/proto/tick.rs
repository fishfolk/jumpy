use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

use crate::prelude::*;

/// Internal counter used for provisioning ticks
static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A tick that can be compared to other ticks from the same server.
///
/// It's essentially a [`u32`] but one that increments every time you create a new [`Tick`] and will
/// loop around to `0` when it gets to `u32::MAX`.
///
/// One tick can be compared with another to find out which one is newer, but because of the
/// wrapping, you can not accurately compare to [`Tick`]'s that are more than `u32::MAX / 2` ticks
/// appart.
///
/// [`AtomicU32`]: std::sync::atomic::AtomicU32
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct Tick(u32);

/// A collection of ticks for other clients
///
/// It's a helper containing the `is_latest()` function to check if a given tick is the latest we've
/// seen for a given client, and update the latest tick for that client if it is.
#[derive(Default)]
pub struct ClientTicks(Vec<Tick>);

impl ClientTicks {
    pub fn is_latest(&mut self, client_idx: usize, tick: Tick) -> bool {
        if client_idx >= self.0.len() {
            let extra_space = client_idx + 1 - self.0.len();
            self.0
                .extend(std::iter::once(Tick::default()).cycle().take(extra_space));
        }

        let current_tick = self.0[client_idx];
        if tick > current_tick {
            self.0[client_idx] = tick;
            true
        } else {
            false
        }
    }
}

impl Tick {
    pub fn next() -> Self {
        COUNTER.fetch_add(1, Relaxed);
        Tick(COUNTER.load(Relaxed))
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl PartialOrd for Tick {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(if self == other {
            std::cmp::Ordering::Equal

        // If `self` is greater than `other`, than we have two scenarios:
        //   - `self` was made later than `other`
        //   - `other` was made later than `self`, but wrapped around to a lower number To account
        //
        // for the second scenario we check what the distance between the two numbers is if we wrapp
        // around from `self` to the `other` across `u32::MAX`. If the difference between the
        // numbers wrapped is greater than the distance between them without wrapping, then we
        // assume that they aren't wrapped, and `other` is actually less than `self.
        } else if self.0 > other.0 && u32::MAX - self.0 + other.0 > u32::MAX / 2 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        })
    }
}

impl Ord for Tick {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

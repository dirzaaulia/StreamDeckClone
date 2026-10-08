use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_secs(60);
const MAX_FAILURES: usize = 20;
const MAX_PEERS: usize = 256;

#[derive(Default)]
pub struct PeerThrottle {
    attempts: HashMap<IpAddr, VecDeque<Instant>>,
}

impl PeerThrottle {
    pub fn allowed(&mut self, peer: IpAddr) -> bool {
        self.prune();
        self.attempts
            .get(&peer)
            .is_some_and(|attempts| attempts.len() < MAX_FAILURES)
            || (self.attempts.len() < MAX_PEERS && !self.attempts.contains_key(&peer))
    }

    pub fn failed(&mut self, peer: IpAddr) {
        self.prune();
        if self.attempts.len() >= MAX_PEERS && !self.attempts.contains_key(&peer) {
            return;
        }
        self.attempts
            .entry(peer)
            .or_default()
            .push_back(Instant::now());
    }

    fn prune(&mut self) {
        self.attempts.retain(|_, attempts| {
            while attempts
                .front()
                .is_some_and(|first| first.elapsed() >= WINDOW)
            {
                attempts.pop_front();
            }
            !attempts.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_failures_block_only_the_source_peer() {
        let mut throttle = PeerThrottle::default();
        let first = IpAddr::from([127, 0, 0, 1]);
        let second = IpAddr::from([127, 0, 0, 2]);
        for _ in 0..MAX_FAILURES {
            assert!(throttle.allowed(first));
            throttle.failed(first);
        }
        assert!(!throttle.allowed(first));
        assert!(throttle.allowed(second));
    }

    #[test]
    fn peer_tracking_is_bounded() {
        let mut throttle = PeerThrottle::default();
        for id in 0..=MAX_PEERS {
            throttle.failed(IpAddr::from([10, 0, (id / 256) as u8, id as u8]));
        }
        assert_eq!(throttle.attempts.len(), MAX_PEERS);
        assert!(!throttle.allowed(IpAddr::from([10, 1, 1, 1])));
        assert!(throttle.allowed(IpAddr::from([10, 0, 0, 0])));
    }
}

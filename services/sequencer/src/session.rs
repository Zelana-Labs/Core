use {dashmap::DashMap, std::net::SocketAddr, zelana_core::AccountId, zelana_net::SessionKeys};

/// Manages active secure sessions for connected clients.
pub struct SessionManager {
    /// Maps IP:Port -> Encryption Keys
    sessions: DashMap<SocketAddr, ActiveSession>,
}

pub struct ActiveSession {
    pub keys: SessionKeys,
    pub account_id: Option<AccountId>, // Known after first valid signature
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    pub fn insert(&self, addr: SocketAddr, keys: SessionKeys) {
        self.sessions.insert(
            addr,
            ActiveSession {
                keys,
                account_id: None,
            },
        );
    }

    pub fn get_mut<F, R>(&self, addr: &SocketAddr, f: F) -> Option<R>
    where
        F: FnOnce(&mut ActiveSession) -> R,
    {
        self.sessions.get_mut(addr).map(|mut entry| f(&mut entry))
    }

    pub fn remove(&self, addr: &SocketAddr) {
        self.sessions.remove(addr);
    }
}

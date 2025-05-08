mod memory;

use std::{collections::HashMap, sync::Arc, sync::Mutex};

use jiff::Zoned;
pub use memory::MemorySessionStore;
use rand::Rng;
use serde::{de::DeserializeOwned, ser::Serialize};

/// A session is a way to store information across requests and associated with
/// visitors.
#[derive(Clone)]
pub struct Session {
    pub id: u128,
    pub data: HashMap<String, serde_json::Value>,
    pub expires_at: Zoned,
    store: Arc<Mutex<Box<dyn SessionStore + Send + Sync + 'static>>>,
}

impl Session {
    /// Creates a new session with a random identifier.
    pub fn new(
        expires_at: Zoned,
        store: Arc<Mutex<Box<dyn SessionStore + Send + Sync + 'static>>>,
    ) -> Self {
        Self {
            id: rand::rng().random(),
            data: HashMap::new(),
            expires_at,
            store,
        }
    }

    /// Get a value from the session data.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        Ok(self
            .data
            .get(key)
            .cloned()
            .map(serde_json::from_value)
            .transpose()?)
    }

    /// Inserts a value into the session data.
    pub fn insert(&mut self, key: &str, value: impl Serialize) -> Result<(), Error> {
        self.data
            .insert(key.to_string(), serde_json::to_value(&value)?);
        self.store.try_lock().unwrap().save(self)?;

        Ok(())
    }

    /// Removes a value from the session data.
    pub fn remove(&mut self, key: &str) -> Result<(), Error> {
        self.data.remove(key).unwrap();
        self.store.try_lock().unwrap().save(self)?;

        Ok(())
    }
}

pub trait SessionStore {
    /// Stores the provided session in the store.
    fn create(&mut self, session: Session) -> Result<(), Error>;

    /// Loads an existing session from the store using the given id.
    fn load(&mut self, id: u128) -> Result<Option<Session>, Error>;

    /// Saves a session to the store.
    fn save(&mut self, session: &Session) -> Result<(), Error>;

    /// Deletes a session from the store using the given id.
    fn delete(&mut self, id: u128) -> Result<(), Error>;
}

/// Represents an error that occurred during session management.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

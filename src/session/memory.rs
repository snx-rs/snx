use crate::session::{Session, SessionStore};

/// A session store that saves sessions to memory.
#[derive(Default)]
pub struct MemorySessionStore {
    data: Vec<Session>,
}

impl SessionStore for MemorySessionStore {
    fn create(&mut self, session: Session) -> Result<(), crate::session::Error> {
        self.data.push(session);

        Ok(())
    }

    fn load(&mut self, id: u128) -> Result<Option<Session>, crate::session::Error> {
        Ok(self
            .data
            .clone()
            .into_iter()
            .find(|session| session.id == id))
    }

    fn save(&mut self, session: &Session) -> Result<(), crate::session::Error> {
        *self.data.iter_mut().find(|s| s.id == session.id).unwrap() = session.clone();

        Ok(())
    }

    fn delete(&mut self, id: u128) -> Result<(), crate::session::Error> {
        let pos = self
            .data
            .clone()
            .into_iter()
            .position(|session| session.id == id)
            .unwrap();
        self.data.remove(pos);

        Ok(())
    }
}

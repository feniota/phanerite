use slotmap::DefaultKey;

/// Deferred structural operations, applied after a system finishes executing.
#[derive(Default)]
pub struct Commands {
    despawns: Vec<DefaultKey>,
}

impl Commands {
    /// Queue an entity for despawn. The entity and all its descendants are removed,
    /// together with their metadata and every aspect, when commands are flushed.
    pub fn despawn(&mut self, key: DefaultKey) {
        self.despawns.push(key);
    }

    pub(crate) fn drain_despawns(&mut self) -> std::vec::Drain<'_, DefaultKey> {
        self.despawns.drain(..)
    }
}

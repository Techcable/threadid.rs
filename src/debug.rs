//! Identifies a thread in a form useful for debugging.

use core::fmt::{Debug, Display, Formatter};

use crate::UniqueThreadId;

/// Identifies a thread in a form useful for debugging.
///
/// Uses the [name] if possible and the id where it is not.
///
/// [name]: std::thread::Thread::name
#[derive(Clone)]
#[must_use]
pub struct DebugThreadId {
    /// This is really an `Arc<ThreadInfo>`,
    /// so it is cheap to Clone and fine if it lives beyond thread death
    info: std::thread::Thread,
    id: UniqueThreadId,
}
impl DebugThreadId {
    /// Get the [`DebugThreadId`] of the current thread.
    ///
    /// Will be significantly slower than [`UniqueThreadId::current`],
    /// due to the need to fetch the thread's name.
    pub fn current() -> DebugThreadId {
        DebugThreadId {
            info: std::thread::current(),
            id: UniqueThreadId::current(),
        }
    }

    /// Get the name of the thread, or `None` if not available.
    #[inline]
    #[must_use]
    pub fn name(&self) -> Option<&'_ str> {
        self.info.name()
    }

    /// Get the id of this thread as a [`UniqueThreadId`].
    #[inline]
    pub fn id(&self) -> UniqueThreadId {
        self.id
    }
}
impl Display for DebugThreadId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.id.to_int())?;
        if let Some(name) = self.name() {
            write!(f, "({name:?})")?;
        }
        Ok(())
    }
}
impl Debug for DebugThreadId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "ThreadId({}", self.id.to_int())?;
        if let Some(name) = self.name() {
            write!(f, ", {name:?})")?;
        } else {
            f.write_str(")")?;
        }
        Ok(())
    }
}
#[cfg(feature = "slog")]
impl slog::Value for DebugThreadId {
    fn serialize(&self, _record: &slog::Record, key: slog::Key, serializer: &mut dyn slog::Serializer) -> slog::Result {
        serializer.emit_arguments(key, &format_args!("{self}"))
    }
}
#[cfg(feature = "serde")]
impl serde::Serialize for DebugThreadId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let name = self.name();
        let mut ser = serializer.serialize_struct("ThreadDebugId", if name.is_some() { 2 } else { 1 })?;
        if let Some(name) = name {
            ser.serialize_field("name", &name)?;
        } else {
            ser.skip_field("name")?;
        }
        ser.serialize_field("id", &self.id())?;
        ser.end()
    }
}

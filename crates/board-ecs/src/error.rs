use std::error::Error;

/// The types of errors used throughout the ECS.
// TODO: Re-evaluate possible errors. Some of them may not be used anymore.
#[derive(Debug, thiserror::Error)]
pub enum EcsError {
    /// A resource was not initialized in the [`World`][crate::World] but the
    /// [`System`][crate::system::System] tries to access it.
    #[error("Resource or component not initialized")]
    NotInitialized,
    /// The requested resource is already borrowed.
    ///
    /// This error is created if the `System` tries to read a resource that has already been mutably
    /// borrowed. It can also happen when trying to mutably borrow a resource that is already being
    /// read.
    ///
    /// This error should not occur during normal use, as the dispatchers can recover easily.
    #[error("Resource or component already borrowed")]
    AlreadyBorrowed,
    /// The execution of the dispatcher failed and returned one or more errors.
    #[error("Dispatcher failed with one or more errors: {0:?}")]
    DispatcherExecutionFailed(Vec<anyhow::Error>),
    /// This variant is for user-defined errors.
    ///
    /// To create an error of this type easily, use the `system_error!` macro.
    #[error("System errored: {0}")]
    SystemError(Box<dyn Error + Send>),
    /// This happens when two Rust types have the same [`TypeUuid`][crate::uuid::TypeUuid], which
    /// must not happen in the same [`World`][crate::world::World].
    #[error(
        "Attempted to initialize resource/component with the same TypeUuid as another type \
        that has already been initialized."
    )]
    TypeUuidCollision,
}

/// The result of a `System`'s execution.
pub type SystemResult = anyhow::Result<()>;

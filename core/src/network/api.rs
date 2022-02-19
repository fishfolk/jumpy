use async_trait::async_trait;

use crate::network::message::NetworkMessage;

use crate::Result;

use super::NetworkEvent;

static mut API_INSTANCE: Option<Api> = None;

pub struct Api {
    backend: Box<dyn ApiBackend>,
}

impl Api {
    fn try_take_instance() -> Option<Api> {
        unsafe { API_INSTANCE.take() }
    }

    #[allow(dead_code)]
    fn try_get_instance() -> Option<&'static mut Api> {
        unsafe { API_INSTANCE.as_mut() }
    }

    #[allow(dead_code)]
    fn get_instance() -> &'static mut Api {
        Self::try_get_instance()
            .unwrap_or_else(|| panic!("Api::get_instance was called before Api::init"))
    }

    pub async fn init<T: 'static + ApiBackend + ApiBackendConstructor>() -> Result<()> {
        unsafe {
            if API_INSTANCE.is_none() {
                let backend = Box::new(T::init().await?);

                API_INSTANCE = Some(Api { backend });
            } else {
                #[cfg(debug_assertions)]
                println!("WARNING: Attempting to initiate api but it is already initiated!");
            }
        }

        Ok(())
    }

    pub async fn close() -> Result<()> {
        if let Some(mut api) = Self::try_take_instance() {
            api.backend.close().await?;

            drop(api);
        }

        Ok(())
    }
}

/// Constructor for backend (needs to be separate from `ApiBackend` so that `ApiBackend` can be
/// object safe
#[async_trait]
pub trait ApiBackendConstructor: Sized {
    /// Init backend
    async fn init() -> Result<Self>;
}

/// This trait should be implemented by all backend implementations
#[async_trait]
pub trait ApiBackend {
    /// Close API connection
    async fn close(&mut self) -> Result<()>;
    /// Dispatch a network message
    fn dispatch_message(&mut self, message: NetworkMessage) -> Result<()>;
    /// Get next event from the queue
    fn next_event(&mut self) -> Option<NetworkEvent>;
}

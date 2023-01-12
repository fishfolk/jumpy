//! Systems and utilities related to specific platform support or platform abstractions

use crate::prelude::*;

use async_channel::{Receiver, Sender};
use bevy::utils::HashMap;
use iyes_loopless::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use native as backend;

#[cfg(target_arch = "wasm32")]
use wasm as backend;

pub struct JumpyPlatformPlugin;

impl Plugin for JumpyPlatformPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Storage>()
            .add_system(load_storage.run_in_state(EngineState::LoadingPlatformStorage));
    }
}

/// Bevy system that will load the [`Storage`] and wait for it to finish loading so it can be used
/// throughout the rest of the game without having to check that storage is loaded.
///
/// Will transition to [`EngineState::LoadingGameData`] when finished.
pub fn load_storage(
    mut started: Local<bool>,
    mut commands: Commands,
    mut storage: ResMut<Storage>,
) {
    // If we haven't started loading
    if !*started {
        debug!("Start loading platform storage");
        // Start loading
        *started = true;
        storage.load();

    // If storage has finished loading
    } else if storage.is_loaded() {
        debug!("Done loading platform storage");
        // Load game
        commands.insert_resource(NextState(EngineState::LoadingGameData));
    }
}

/// The type of the inner data in [`Storage`]
type StorageData = HashMap<String, serde_yaml::Value>;

/// Resource for accessing platform specific persistent storage apis through a simple interface.
#[derive(Resource)]
pub struct Storage {
    /// The in-memory storage data that we operate on when getting and setting values.
    data: Option<StorageData>,
    /// A data receiver that gets set when we are awaiting the result of a [`Self::load()`] operation.
    data_receiver: Option<Receiver<StorageData>>,
    /// The sender we use to send storage requests to the storage backend
    backend_sender: Sender<StorageRequest>,
}

impl FromWorld for Storage {
    fn from_world(_: &mut World) -> Self {
        let backend_sender = backend::init_storage();

        Self {
            data: None,
            data_receiver: None,
            backend_sender,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("Storage has not been loaded yet")]
    NotLoaded,
    #[error("Storage backend connection lost")]
    BackendLost,
    #[error("Storage key could not be serizlized/deserialized: {0}")]
    SerializationError(#[from] serde_yaml::Error),
}

impl Storage {
    fn check_pending_data_load(&mut self) {
        // If we are waiting on a data load response
        if let Some(receiver) = &mut self.data_receiver {
            // If the data has been loaded
            if let Ok(data) = receiver.try_recv() {
                // Set the local data and clear the load receiver
                self.data = Some(data);
                self.data_receiver = None;
            }
        }
    }

    /// Get whether or not storage has been loaded.
    ///
    /// Before you may get or set values, you must [`load()`][Self::load] the storage.
    pub fn is_loaded(&mut self) -> bool {
        self.check_pending_data_load();

        self.data.is_some()
    }

    /// Load from platform storage into memory.
    ///
    /// This process is asynchronous. Loaded data will not be available immediately, and
    /// [`Self::is_loaded()`] can be used to check whether or not data has been loaded.
    pub fn try_load(&mut self) -> Result<(), StorageError> {
        let (result_sender, data_receiver) = async_channel::unbounded();

        self.data_receiver = Some(data_receiver);

        self.backend_sender
            .try_send(StorageRequest::Load { result_sender })
            .map_err(|_| StorageError::BackendLost)?;

        Ok(())
    }

    /// Load from platform storage into memory.
    ///
    /// This process is asynchronous. Loaded data will not be available immediately, and
    /// [`Self::is_loaded()`] can be used to check whether or not data has been loaded.
    ///
    /// # Panics
    ///
    /// Panics if the storage backend disconnects for some reason.
    #[track_caller]
    pub fn load(&mut self) {
        self.try_load().expect("Storage load")
    }

    /// Try to get a value from the in-memory storage cache.
    pub fn try_get<T>(&mut self, key: &str) -> Result<Option<T>, StorageError>
    where
        T: Serialize + DeserializeOwned,
    {
        self.check_pending_data_load();

        if let Some(data) = &self.data {
            let value = data.get(key).cloned();

            if let Some(value) = value {
                let value = serde_yaml::from_value(value)?;

                Ok(Some(value))
            } else {
                Ok(None)
            }
        } else {
            Err(StorageError::NotLoaded)
        }
    }

    /// Get a value from the in-memory storage cache.
    ///
    /// # Panics
    ///
    /// This will panic if storage has not been loaded yet or if there is a deserialization error.
    #[track_caller]
    pub fn get<T>(&mut self, key: &str) -> Option<T>
    where
        T: Serialize + DeserializeOwned,
    {
        match self.try_get(key) {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "Error deserializing storage key, returning empty value: {}",
                    e
                );
                None
            }
        }
    }

    /// Set a value in the in-memory storage cache.
    ///
    /// Changes will not be persisted until [`Self::save()`] is called.
    pub fn try_set<T>(&mut self, key: &str, value: &T) -> Result<(), StorageError>
    where
        T: Serialize + DeserializeOwned,
    {
        self.check_pending_data_load();

        if let Some(data) = &mut self.data {
            let value = serde_yaml::to_value(value)?;
            data.insert(key.into(), value);

            Ok(())
        } else {
            Err(StorageError::NotLoaded)
        }
    }

    /// Set a value in the in-memory storage cache.
    ///
    ///
    /// Changes will not be persisted until [`Self::save()`] is called.
    ///
    /// # Panics
    ///
    /// This will panic if storage has not been loaded yet or if there is a serialization error.
    #[track_caller]
    pub fn set<T>(&mut self, key: &str, value: &T)
    where
        T: Serialize + DeserializeOwned,
    {
        self.try_set(key, value).expect("Set value in storage")
    }

    /// Saves the in-memory storage cache to persistent storage.
    ///
    /// This operation is asynchronous and returns a [`SaveTask`] that can be used to check when the
    /// operation is complete.
    pub fn try_save(&mut self) -> Result<SaveTask, StorageError> {
        self.check_pending_data_load();

        if let Some(data) = &self.data {
            let (result_sender, result_receiver) = async_channel::unbounded();

            self.backend_sender
                .try_send(StorageRequest::Save {
                    data: data.clone(),
                    result_sender,
                })
                .map_err(|_| StorageError::BackendLost)?;

            Ok(SaveTask(result_receiver))
        } else {
            Err(StorageError::NotLoaded)
        }
    }

    /// Saves the in-memory storage cache to persistent storage.
    ///
    /// This operation is asynchronous and returns a [`SaveTask`] that can be used to check when the
    /// operation is complete.
    ///
    /// # Panics
    ///
    /// This will panic if the storage hasn't been loaded yet or if the storage backend disconnects
    /// for some reason.
    #[track_caller]
    pub fn save(&mut self) -> SaveTask {
        self.try_save().expect("Save storage")
    }
}

/// [`Storage::save()`] task handle that can be used to check whether or not saving has been
/// completed.
pub struct SaveTask(Receiver<()>);

impl SaveTask {
    /// Get whether or not saving has been completed.
    #[allow(unused)] // TODO: Remove when we use it
    pub fn is_complete(&mut self) -> bool {
        !self.0.is_empty()
    }
}

enum StorageRequest {
    Load {
        result_sender: Sender<StorageData>,
    },
    Save {
        data: StorageData,
        result_sender: Sender<()>,
    },
}

/// Native platform support
#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::{
        fs,
        io::{Read, Write},
    };

    use async_channel::Sender;
    use bevy::{
        prelude::{error, trace},
        tasks::IoTaskPool,
        utils::HashMap,
    };

    use super::StorageRequest;

    pub(super) fn init_storage() -> Sender<StorageRequest> {
        trace!("Initialize platform storage backend");
        let io_task_pool = IoTaskPool::get();

        // Create channel used for sending and receving storage requests
        let (sender, receiver) = async_channel::unbounded();

        // Identify project storage file path
        let project_dirs = directories::ProjectDirs::from("org", "FishFolk", "Jumpy")
            .expect("Identify system data dir path");
        let file_path = project_dirs.data_dir().join("storage.yml");

        trace!(?file_path, "Platform storage filepath");

        // Spawn an async task that will read and write to the filesystem
        io_task_pool
            .spawn(async move {
                while let Ok(request) = receiver.recv().await {
                    match request {
                        StorageRequest::Load { result_sender } => {
                            let data = if file_path.exists() {
                                let mut file = fs::OpenOptions::new()
                                    .read(true)
                                    .open(&file_path)
                                    .expect("Open storage file");

                                let mut contents = Vec::new();
                                file.read_to_end(&mut contents).expect("Read storage file");

                                match serde_yaml::from_slice(&contents) {
                                    Ok(data) => data,
                                    Err(e) => {
                                        error!(
                                            "Error deserializing storage file. Ignoring existing \
                                            data and overwriting on next attempt to save: {}",
                                            e
                                        );
                                        Default::default()
                                    }
                                }
                            } else {
                                HashMap::new()
                            };

                            result_sender.try_send(data).ok();
                        }
                        StorageRequest::Save {
                            data,
                            result_sender,
                        } => {
                            let data = serde_yaml::to_string(&data).expect("Serialize data");
                            if let Some(parent) = file_path.parent() {
                                std::fs::create_dir_all(parent).expect("Create storage dir");
                            }
                            let mut file = fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open(&file_path)
                                .expect("Open storage file");

                            file.write_all(data.as_bytes()).expect("Write storage file");

                            result_sender.try_send(()).ok();
                        }
                    }
                }
            })
            .detach();

        sender
    }
}

/// WASM platform support
#[cfg(target_arch = "wasm32")]
mod wasm {
    use async_channel::Sender;
    use bevy::{prelude::*, tasks::IoTaskPool};

    use super::StorageRequest;

    const BROWSER_LOCAL_STORAGE_KEY: &str = "jumpy-platform-storage";

    /// Initialize storage backend
    pub(super) fn init_storage() -> Sender<StorageRequest> {
        trace!("Initialize platform storage backend");
        let io_task_pool = IoTaskPool::get();

        // Create channel used for sending and receving storage requests
        let (sender, receiver) = async_channel::unbounded();

        // Spawn an async task for interfacing with browser local storage
        io_task_pool.spawn(async move {
            let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();

            // Loop as long as there are still storage request senders in scope
            while let Ok(request) = receiver.recv().await {
                match request {
                    StorageRequest::Load { result_sender } => {
                        let data = local_storage
                            .get_item(BROWSER_LOCAL_STORAGE_KEY)
                            .unwrap()
                            .and_then(|data| match serde_yaml::from_str(&data) {
                                Ok(data) => data,
                                Err(e) => {
                                    error!(
                                        "Error deserializing storage, ignoring existing data \
                                        and overwriting on next attempt to set data in storage: {}",
                                        e
                                    );

                                    None
                                }
                            })
                            .unwrap_or_default();

                        result_sender.try_send(data).ok();
                    }
                    StorageRequest::Save {
                        data,
                        result_sender,
                    } => {
                        let data = serde_yaml::to_string(&data).expect("Serialize platform data");

                        local_storage
                            .set_item(BROWSER_LOCAL_STORAGE_KEY, &data)
                            .unwrap();

                        result_sender.try_send(()).ok();
                    }
                }
            }
        });

        sender
    }
}

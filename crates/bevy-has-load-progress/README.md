Helper plugin and derive macro for tracking the load progress of nested structs containing asset
handles.

This crate exposes a trait [`HasLoadProgress`] that may be derived on structs that contain Bevy
asset [`Handle`]s. The idea is that you may have a struct with asset handles contained somewhere
inside, maybe deeply nested or stored in vectors, etc., and you need to be able to get the load
progress of _all_ of the handles inside that struct.

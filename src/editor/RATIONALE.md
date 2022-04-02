TODO: Delete this file before merging!

- I changed the type of selected_tool from an option of type ID to an enum, because the available
tools are known at compiletime.
- `CreateLayerWindow` does not allow undoing, only the main editor context does. As such, its `ui`
method doesn't need to be immutable, because the changes done to the window are irrelevant to the
history and can occur without having to keep track of them. (PD: TextEdit changes are kept track of
internally by egui and can be undone/redone without any external code)
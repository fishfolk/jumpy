TODO: Delete this file before merging!

- I changed the type of selected_tool from an option of type ID to an enum, because the available
tools are known at compiletime.
- `CreateLayerWindow` does not allow undoing, only the main editor context does. As such, its `ui`
method doesn't need to be immutable, because the changes done to the window are irrelevant to the
history and can occur without having to keep track of them. (PD: TextEdit changes are kept track of
internally by egui and can be undone/redone without any external code)
- The `actions` module file was too crowded, so I split all actions into their own files in `actions/`.
I think it's much more organized this way.
- Actions have been stripped from the `Action` suffix, as the parent module already tells us that
they are actions: `editor::actions::CreateLayerAction`. So instead of importing all the actions from
the editor and directly calling them with their names, I just import the actions module and refer to
them as `actions::CreateLayer` for instance. I think this is less redundant and also eliminates the
need to import all used actions, as their names are shorter.
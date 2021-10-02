# Editor

This is a quick overview of the **work in progress** editor, in its current state.

## Actions

The editor is driven by actions. In the `actions` module, there is an enum, named `EditorAction` that holds all
the available actions. These might be simple things, like UI changes or, if they alter data; implementations of
`UndoableAction`. Not all UI actions are implemented as `EditorActions` but if the alternative is code repetition
or if the action is performed by another abstraction level, it should be added to the enum.

Any action that modifies data, is implemented as an `UndoableAction`.

## Contributing

The framework has already been laid out, and it is expected that functionality that is proposed follow this.

This means that any actions that modify map data should be `UndoableAction` trait implementations and that these
implementations stores the state of the data it modifies, before application, and reinstates it, exact, when the
actions `undo` method is called. There can be no exceptions to this.

Furthermore, actions should be called from the existing ui, and the existing ui should not be expanded, without
it being approved beforehand. This is to reduce clutter, which can break a tool like this.
The exception to this, are windows that are needed to input parameters to an action, like the `CreateLayerWindow`.
If adding such windows, use the `WindowBuilder` and follow the general workflow of the rest of the code. The design
is not final but if everything is made coherently, implementing design changes will be less time consuming.
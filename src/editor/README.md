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
it being discussed beforehand in an issue or on discord. We want to keep the UI simple and straight forward, so
any additions or changes should be planned accordingly.

It is also important to make sure that you adhere to the API that has been created for UI elements. This means
that you should implement the `Window` trait to create windows, the `ToolbarElement` trait to expand the toolbars,
and so on.
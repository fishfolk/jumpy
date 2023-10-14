# Jumpy

Jumpy is a pixel-style, tactical 2D shooter with a fishy theme.

This is the project's internal developer API and architecture documentation. We want these docs to
be the go-to place for understanding how Jumpy works, so you'll find much more than the comments on
various types and functions.

#### 🚧 Under Construction

Jumpy is still under heavy development and the docs as well as the code will be changing and
incomplete. If you want to help out, feel free to fill in some missing docs or clarify something we
didn't explain very well. We're glad to answer any questions you might have along the way!

#### Diagrams

Throughout the docs there are diagrams to explain certain topics. These may contain clickable links,
highlighted in blue, that will bring you to the relevant module or struct. We also use a convention to
indicate what kind of code object it links to, based on the shape of the box.

<pre class="mermaid">
graph LR
  ex("Example Link ( Note )"):::dotted -.-> sm(GameMeta):::code
  click sm call docLink(jumpy/struct.GameMeta.html)

  crate:::code --> module([module]):::code --> struct(struct):::code --> concept>Concept]
</pre>

You can pan the diagram by clicking and dragging or zoom by holding `ctrl` and scrolling.

[gh_issue]: https://github.com/fishfolk/jumpy/issues/new

## Overall Architecture

Jumpy has just finished a migration to the new [`bones_framework`] and some of the docs might not
be up-to-date yet.

The good news is that everything is simpler now! All of jumpy has been moved to one crate with many
of it's more foundational features being moved into [`bones_framework`]. Additionally, Jumpy no longer
uses Bevy or the Bevy ECS directly. The game is fully built on Bones with Bevy being used in the
background, but only for rendering.

An explanation post and more up-to-date docs are comming soon!

[`bones_framework`]: https://fishfolk.github.io/bones/rustdoc/bones_framework/index.html

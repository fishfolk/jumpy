# Jumpy

Jumpy is a pixel-style, tactical 2D shooter with a fishy theme.

This is the project's internal developer API and architecture documentation. We want these docs to
be the go-to place for understanding how Jumpy works, so you'll find much more than the comments on
various types and functions.

#### 🚧 Under Construction

Jumpy is still under heavy development and the docs as well as the code will be changing and
incomplete. If you want to help out, feel free to fill in some missing docs or clarify something we
didn't explain very well. We're glad to answer any questions you might have along the way!

#### Organization

Each of the modules in the list below contains it's own docs, which should explain details of what
is contained in that model and how it works. Some of these may be simple, and others, such as the
[`networking`] module have guide-level explanations on that facet of the game.

Feel free to explore whatever seems interesting to you, and please [open an issue][gh_issue] if you
feel like something is missing, or would like further guidance on a particular topic!

##### Diagrams

Throughout the docs there are diagrams to explain certain topics. These may contain clickable links, highlighted in blue, that will bring you to the relevant module or struct. We also use a convention to indicate what kind of code object it links to, based on the shape of the box.

<pre class="mermaid">
graph LR
  ex("Example Link ( Note )"):::dotted -.-> sm(SessionManager):::code
  click sm call docLink(jumpy/session/struct.SessionManager.html)

  crate:::code --> module([module]):::code --> struct(struct):::code --> concept>Concept]
</pre>

You can pan the diagram by clicking and dragging or zoom by holding `ctrl` and scrolling.

[gh_issue]: https://github.com/fishfolk/jumpy/issues/new

## Overall Architecture

There are a few major crates that are used in Jumpy:

- [Jumpy Core][jumpy_core] - Core Gameplay Logic
- [Bones](https://fishfolk.org/development/bones/introduction/) - Core Entity Component System & Rendering Components
- [Bevy](https://bevyengine.org) - Rendering, Asset Loading, and User Input

<pre class="mermaid">
graph TD
  subgraph jumpy
    direction TB

    SessionManager(SessionManager):::code
    bevyInput --> SessionManager

    click SessionManager call docLink(jumpy/session/struct.SessionManager.html)
    ui([ui]):::code
    click ui call docLink(jumpy/ui/index.html)
    editor([editor]):::code
    click editor call docLink(jumpy/ui/editor/index.html)
    localization([localization]):::code
    click localization call docLink(jumpy/localization/index.html)
    networking([networking]):::code
    click networking call docLink(jumpy/networking/index.html)

    subgraph jumpy_core
      CoreSession(CoreSession):::code
      click CoreSession call docLink(jumpy_core/session/struct.CoreSession.html)
      gameplaySystems>Gameplay Systems]
      CoreSession --> gameplaySystems
    end
    SessionManager --> CoreSession
    SessionManager -.- networking

    gameplaySystems -- Update --> World

    subgraph bones
      World(World):::code
      click World href "https://fishfolk.github.io/bones/rustdoc/bones_ecs/struct.World.html"
      renderingComponents("Rendering / Audio
          Components"):::dotted
      renderingComponents -.- World
    end

    World -- Sound State --> bevy_kira_audio:::code
    World -- World State --> bones_bevy_renderer:::code
    click bevy_kira_audio href "https://docs.rs/bevy_kira_audio/latest/bevy_kira_audio/"
    click bones_bevy_renderer href "https://fishfolk.github.io/bones/rustdoc/bones_bevy_renderer/index.html"

    ui --> bevyRenderer
    editor -.- ui
    localization -.- ui

    bones_bevy_renderer --> bevyRenderer

    subgraph bevy
      bevyInput[\"User Input
      Assets"/]
      bevyRenderer[/Renderer\]
    end
  end
</pre>

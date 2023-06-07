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

Throughout the docs there are diagrams to explain certain topics. These may contain clickable links, highlighted in blue, that will bring you to the relevant module or struct. For example:

<pre class="mermaid">
graph LR
  sm(SessionManager):::code
  click sm call docLink(jumpy/session/struct.SessionManager.html)
  ex(Example Link) -.-> sm
</pre>

You can also pan by clicking and dragging, and zoom by holding `ctrl` and scrolling.

[gh_issue]: https://github.com/fishfolk/jumpy/issues/new

## Overall Architecture

You can explore the documentation for each of the modules below to get more details on what each
one does. Here we will outline the high-level architecture.

There are a few major crates that are used in Jumpy:

- [Bevy](https://bevyengine.org)
- [Bones](https://fishfolk.org/development/bones/introduction/)
- [Jumpy Core][jumpy_core]

The overall architecture is depicted in the diagram below.

<pre class="mermaid">
graph TB
  sm -- Input / Create / Snapshot / Restore --> cs
  subgraph jumpy
    direction TB
    sm(SessionManager):::code
    click sm call docLink(jumpy/session/struct.SessionManager.html)
    ui(UI)
    ed(Editor)
    lo(Localization)
    n(networking):::code
    click n call docLink(jumpy/networking/index.html)
    bkr[Bevy Kira Audio]
    bbr[Bones Bevy Renderer]
    bbr --> br
    sm -.- n

    subgraph jumpy_core
      cs(CoreSession):::code
      click cs call docLink(jumpy_core/session/struct.CoreSession.html)
      gp>Gameplay Systems]
      cs --> gp

    end
    gp -- Update --> ecs

    ecs -- Sound State --> bkr
    ecs -- World State --> bbr
    subgraph bones
      ecs(World):::code
      click ecs href "https://fishfolk.github.io/bones/rustdoc/bones_ecs/struct.World.html"
      style rc stroke-dasharray: 5 5
      rc("Rendering / Audio
          Components")
      rc -.- ecs
    end

    ui --> br
    ed -.- ui
    lo -.- ui
    bi --> sm
    subgraph bevy
      br(Renderer)
      bi(User Input)
    end
  end
</pre>

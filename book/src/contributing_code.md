# Contributing Code to Fish Fight

Proposing code changes has 3 main steps:

1. Setting up your development environment
2. Pick an issue to work on
3. Write code and submitting for review

## 1. Setting up your development environment

### Before starting

Make sure [Rust](https://www.rust-lang.org/) is installed with [Rustup.rs](https://rustup.rs/).

Have an account on [GitHub](https://github.com/): this is where you'll find the [source code](https://github.com/fishfight/FishFight) for Fish Fight.

### Getting the source code

Fish Fight uses [git](https://git-scm.com/) as its established choice of version control. You'll be
using it to clone, track, and manage the source code locally on your machine.

To get a copy of the code, you'll first need to [fork the repository](https://docs.github.com/en/get-started/quickstart/fork-a-repo).
The GitHub repository for Fight Fight is [available here](https://github.com/fishfight/FishFight).
This will allow you to make changes to the project without affecting the original one.

Once you've forked the Fish Fight repository, you can now clone it:

`git clone https://github.com/YOUR_ACCOUNT_NAME/FishFight.git`

It's also possible to clone the repo using SSH or GitHub CLI. For more information on how to do
this, see the [official GitHub documentation](https://docs.github.com/en/get-started/quickstart/fork-a-repo#cloning-your-forked-repository).

Depending on your connection, the clone can take around 1 minute.

By the end of this step, you should have successfully forked and downloaded a clone of Fish Fight on your machine.

### Build and run

You can now build your forked copy of Fish Fight:

`cargo build`

This process should take about a minute or two depending on your machine. You can also **build and run** the game with a single command:

`cargo run`

## 2. Finding a good first issue

Now that you can build and run Fish Fight source code, let's find something to work on!
We recommend all newcomers start with our [Development Tracks](https://github.com/fishfight/FishFight/issues/124).
You can also browse [project's issues list](https://github.com/fishfight/FishFight/issues) and pick something with a [help wanted label](https://github.com/fishfight/FishFight/labels/help%20wanted).
In general, you can comment on an issue expressing interest and someone will assign it to you.

Additionally, if there's a track or issue you're particularly interested in, but you don't know where to start, feel free to
reach out to the [Fish Fight Discord community](https://discord.gg/4smxjcheE5) with your questions
and someone should reach out shortly!

## 3. Write code and submitting for review

In general, Fish Fight uses a branch-based workflow (or [GitHub flow](https://docs.github.com/en/get-started/quickstart/github-flow)) for submitting changes to the project.

### Create a new branch

You'll want to [create a new branch](https://docs.github.com/en/get-started/quickstart/github-flow#create-a-branch) off `main`:

`git checkout -b <branch_name> main`

You'll replace `<branch-name>` with something short and descriptive. For example, if you're adding a new
item to Fish Fight, your branch name might look like this:

`git checkout -b add_new_weapon main`

### Commit your changes

Once you've made the desired changes to the code and you're ready for someone on the Fish Fight
team to review, you need to [commit your work](https://git-scm.com/docs/git-commit). But first,
we have to run a few commands to ensure the code you're submitting is properly formatted:

1. `cargo clippy -- -W clippy::correctness -D warnings`
2. `cargo fmt`

Now we can start committing your work. First, stage your changes:
`git add`

Now commit. It's always good practice to provide a short message that details what the changes are. For example:
`git commit -m "Add a new weapon"`

### Submitting for review

You can now start submitting your work for review. First, push your changes:

`git push`

This will create a new branch on your forked copy of Fish Fight. You can then proceed [making a pull request](https://docs.github.com/en/desktop/contributing-and-collaborating-using-github-desktop/working-with-your-remote-repository-on-github-or-github-enterprise/creating-an-issue-or-pull-request#creating-a-pull-request).
This is how the Fish Fight team will review and provide feedback on your work.

[More information on contributing using GitHub](https://docs.github.com/en/desktop/contributing-and-collaborating-using-github-desktop).

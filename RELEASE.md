# Creating a Release

[GitHub releases](https://github.com/fishfolk/jumpy/releases) are automated via [GitHub actions](./.github/workflows/release.yml) and triggered by pushing a tag.

1. Bump the version in [Cargo.toml](Cargo.toml) and [core/Cargo.toml](core/Cargo.toml).
2. Update [Cargo.lock](Cargo.lock) by building the project. (`cargo build`)
3. Commit and push the changes. (i.e. submit a pull request)
4. Either [draft a new release](https://github.com/fishfolk/jumpy/releases/new) from the GitHub interface or create a new tag and push it via the command line.
5. While naming the release, do not include the version in the title. (e.g. "Level Editor")
6. Add release notes and highlights.

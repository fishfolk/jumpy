# How to add a level to the Fish Folk: Jumpy

Tiled: https://www.mapeditor.org/, also in most distros is available as `tiled` package. 
1.7+ version is required, or you will have to open your `json` map file in a text editor and change the maps `version`
field to a `string`, manually, in order for it to work.

## Project window

Tiled projects are an optional thing to help navigate the levels while in tiled.

To open the project toolbar, click:
`View -> Toolbars -> Project`

To open the project with all our levels:
`File -> Open file or project -> levels.tiled-project`

This should result in a list of all the available maps in your project toolbar (the left-most list view)

## Creating a level

Section heavily under construction, if you feel there is some tips that can help designing levels - just throw them here, will reformat and make it nice later!

Usually to create a new level I click "Right Click -> Open containing folder" on lev*.json in Project window and in the file manager I duplicate this file. 

If you do not copy an existing map, there are a few requirements for Tiled maps that need to be met, in order to make them compatible with our internal format:

- Every tileset needs a `String` prop, named `texture_id`, that references a texture in `assets/textures.json`. This texture entry also needs to be of type `tilemap`.
The default ones, used in the core maps, are `default_tileset`, for the tiles, and `default_decoration` for the decorations (decorations are objects, not tiles, though. More on that below).
- Every tile layer with collisions needs a `bool` prop, named `collision`, set to true.
- Every object needs both a `name` and a `type` set (except for spawn points, which only need a `type`). The `name` acts as an id in-game, which will reference an entry in the appropriate data file, based on the objects `type`. Available types are:
    * `item` for items such as power-ups and weapons 
    * `decoration` for decorations, such as `seaweed` and `pots`, found in most maps
    * `spawn` for player spawn points

## Adding a map to the game

Add an entry to `assets/maps.json` and it will show up in the in-game maps menu:

```json
{
  "name": "My Awesome Map",
  "path": "assets/levels/lev01.json",
  "preview": "assets/levels/lev01.png",
  "is_tiled": true
}
```

Remember to set `is_tiled` to `true` for any map made in Tiled, so that the game knows that it must be converted to the internal format when it is loaded.

If you have no preview for your map, yet, you can use the generic `no_preview.png` in stead. This should not be used outside of testing, though.

This is it, after adding level to `levels.json`, it should appear in the game!
Levels are sorted in the same order as they are in the toml.

## Making a PR

To put level back into the game - level should make it to our git repo. 

The easiest way to do it is making a pull request.

I guess there are some UI clients for git, feel free to add some guides right here :)

but the idea is: 

Check what files were changed
```bash
> git status

On branch master
Your branch is up to date with 'origin/master'.

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
	assets/levels/levels.toml

Untracked files:
  (use "git add <file>..." to include in what will be committed)
	assets/levels/lev07.json
	assets/levels/lev07.png
```

This mean that "levels.toml" was changed and lev07 is a new level unknown to git.

Add all the files that should go to the repo:
```bash
git add asssets/levels
```

Check status again: 
```bash
On branch master
Your branch is up to date with 'origin/master'.

Changes to be committed:
  (use "git restore --staged <file>..." to unstage)
	new file:   assets/levels/lev07.json
	new file:   assets/levels/lev07.png
	modified:   assets/levels/levels.toml
```

Looks good enough, then to actually make a PR:

Create a named branch 
```bash
git switch -m "assets_lev07"
```
Commit changes
```bash
git commit -m "I made a level!"
```

## Pushing to GitHub

- Fork a repo in github's web UI
- Push to your repo `git push git://github.com/YOUR_USERNAME/fish2`
- Click the green button that will appear on your fork of github web ui

And there is no way to break anything in the main repo doing this, so dont be afraid. It's OK to commit extra files, do tons of extra commits, commit with a crazy commit message etc. 

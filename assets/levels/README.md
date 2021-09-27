# How to add a level to the FishFight

Tiled: https://www.mapeditor.org/, also in most distros is available as `tiled` package. 
1.7+ version required. 

## Project window

![image](https://user-images.githubusercontent.com/910977/126675836-128e394c-755d-4061-9103-5ed93f6e55cd.png)
*"Project" is that little thing on the left.*

Tiled projects are an optional thing to help navigate the levels while in tiled.

To open project window, click:
`View -> Toolbars -> Project`

To open project with all our levels:
`File -> Open file or project`

Then click on lev0*.json in the project window to open up a level. 

## Creating a level

Section heavily under construction, if you feel there is some tips that can help designing levels - just throw them here, will reformat and make it nice later!

Usually to create a new level I click "Right Click -> Open containing folder" on lev*.json in Project window and in the file manager I duplicate this file. 

Right now objects positioning is kinda messed up and awaits fixing, so item will be spawn somewhere around that thing in "items" layer, but exact positioning of things like swords is pretty much impossible right now. Will fix!

## Adding level to the game

Add a record to `assets/levels/levels.toml`, just like this: 
```toml
[[level]]
map = "assets/levels/lev01.json"
preview = "assets/levels/lev01.png"
```

**Note** that both map and preview are required! If there is no preview yet - just use a random texture, preview of other map or whatever.

In the future there will be some other metadata, description, name, etc. 
Feel free to add keys to the toml like

```toml
name = "Cool level name"
description = "Sword fighting 1x1 level for the real fishes"
```

They will do nothing in the game, but may be helpful to figure what UI we will need in level selection menu etc. 

This is it, after adding level to levels.toml it should appear in the game!
Levels are sorted in the same order as they are in the toml. 
Maybe we need a "priority" toml param? Maybe later, rn just move records in the toml around


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
- Click the green button that will appear on your fork of GH web ui

And there is no way to break anything in the main repo doing this, so dont be afraid. It OK to commit extra files, do tons of extra commits, commit with a crazy commit message etc. 

## Pushing to SourceHut

SourceHut, right now, is a back-up option and  PR discussion is going to happen on GitHub. However there is a way to both clone a repo and do a PR through SourceHut.

- For a repo with "Clone repo to your account button"
- Push your changes to your SourceHut account 
- Click the "Prepare changeset" button in SourceHut
- Click to the earliest commit you want to be in the PR (all your commits should be ble)
- And send it to "not.fl3@gmail.com". I will migrate it to GH.

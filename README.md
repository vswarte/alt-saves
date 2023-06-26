# Elden Ring alternative saves

**A word of warning**: this mod has been tested as far as I can but mod loaders use wildly different approaches and timings
for loading DLLs. Make sure to backup your save games because if this mod loads too late it might start writing to your
vanilla save file.

This mod has been extensively tested with [ModEngine2](https://github.com/soulsmods/ModEngine2).

#### How do I back up my save files?
You can find your save files in the folder `%appdata%/EldenRing/<steam ID>`. Making a copy of that folder should suffice.

## What is this?
This mod allows you to use an alternate savefile for your playthrough when enabled.

## Why do I want this?
Maybe you're a user with multiple mods and want to simplify the process of going back to vanilla online for a few.
(afaik this currently requires swapping out save files manually or just playing modded with seamless coop).
Or maybe you're a mod developer and you want your overhaul mod to use a different extension so people don't accidentally
load save files that are affected by other mods.

## How does it work?
It hooks whatever is responsible for reading from and writing to your save files intercepting the file extension that
is used.

## Scenarios:

### I am a player and have a set of mods that I don't want touching my vanilla savefile.
Place the DLL in your modengine2 folder or any other folder, then load the DLL modengine2 using the following
configuration options:

config_eldenring.toml:
```toml
# ...
external_dlls = ["eldenring_alt_saves.dll", "some_other_mod.dll"]
#...
```
*If you've chosen a path outside your modengine2 directory make sure to put in the appropriate path to the DLL.*

This will make the DLL assume extension `.mod` as opposed to `.sl2` for your vanilla saves, for seamless coop it will
use `.mod.co2` instead of `.co2`.

### I am a mod developer and want my mod to use a different save location
Place the DLL in your mod folder, then load the DLL modengine2 using the following configuration options:
config_eldenring.toml:
```toml
# ...
external_dlls = ["mod/eldenring_alt_saves.dll", "mod/some_other_dlls.dll"]
#...
```

Then read to the configuration section.

## Configuration
Sometimes you have multiple sets of mods, or you're developing a mod that should use a different save file by default.
Either way you need to tweak the extension(s).

This is achieved by supplying a non-mandatory config file for the altsaves DLL. You'll need to create a config file
called `altsaves.toml`. The contents should look like this:
```toml
# Extension you want to use for the save files
extension = ".my-extension-goes-here"

# Extension you want use for the save files while playing with seamless coop enabled.
seamless_extension = ".my-extension-goes-here.co2"
```

The `altsaves.toml` can be put in the game directory immediately (recommended approach for users with manually managed 
sets of mods) *or* the modengine2 mod folder itself (allows mod developers to determine the save file extensions).

### Best practices around configuration (read me!!!)
If you are setting up this mod for yourself and you are using the config, make sure to specify an alternative seamless
coop extension. **Not doing so will make modded seamless and modded non-seamless playthroughs use the same file**.


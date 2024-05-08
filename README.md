![Mod.io For Godot](https://github.com/aNaOH/modio-godot/blob/main/logo.svg?raw=true)

The main branch contains the lastest working version of the plugin, the master branch contains the lastest version of the code, even if it not works or if I'm stuck with something.

Adds mod.io integration for Godot using GDExtension.

Currently supports:

 - Windows
 - OSX
 - Linux

**TODO**

 - Upload mods
 - Paging

## How to use

    var modio = ModIO.new()
	modio.connect(api_key, game_id)
	var mods = modio.get_mods(query)
get_mods() returns an array of dictioraries, the dictionary is formed by:

 - id
 - date_updated
 - date_live
 - profile_url
 - modfile_url
 - modfile_name
 - modfile_size
 - tags

get_mods() needs a string argument for the search query, if you want to list all mods just use "" as the argument

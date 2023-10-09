![Mod.io For Godot](https://github.com/aNaOH/modio-godot/blob/main/logo.svg?raw=true)

Adds mod.io integration for Godot using GDExtension.
Currently supports:

 - Windows

**TODO**

 - Support for OSX
 - Support for Linux
 - Support for Android
 - Upload mods
 - Paging

## How to use

    var modio = ModIO.new()
	modio.connect(api_key, game_id)
	var mods = modio.get_mods(query)
get_mods() returns an array of dictioraries, the dictionary is:

 - id
 - date_updated
 - date_live
 - profile_url
 - modfile_url
 - modfile_name
 - modfile_size
 - tags

get_mods() needs an string argument for the search query, if you want to list all mods just use "" as the argument

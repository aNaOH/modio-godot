
![Mod.io For Godot](https://github.com/aNaOH/modio-godot/blob/main/logo.svg?raw=true)

The main branch contains the lastest working version of the plugin, the 'latest' branch contains the latest version of the code, even if it not works or if I'm stuck with something.

Adds mod.io integration for Godot using GDExtension.

Currently supports:

- Windows
- OSX
- Linux

# How to initialize

First, you have to get an API key for your game at https://mod.io/g/your-game/admin/api-key (if you don't have a game page on mod.io, get it [here](https://mod.io/g/add/)) alongside with the game ID on the section API path from the same page: https://g-gameid.modapi.io/v1

    var modio = ModIO.new()
	modio.connect(api_key, game_id)

# Functions

## Basic

### get_mods(query : String, page : int, per_page : int)

get_mods() returns an array of dictioraries, the dictionary is formed by:

- id
- name (Thanks to @d10sfan for adding it!)
- submitter (Thanks to @d10sfan for adding it!)
- summary
- description
- date_updated
- date_live
- profile_url
- modfile_url
- modfile_name
- modfile_size
- tags

get_mods() needs a string argument for the search query, if you want to list all mods just use "" as the argument

### upload_mod(user_token: String, modfile_path: String, name: String, summary: String, thumbnail_path: String)

Needs a token obtained using one of the auth functions
Compresses the file at modfile_path onto a zip file and uploads it to mod.io with the specified name and summary, also uploads the thumbnail at thumbnail_path (This image has to have an aspect ratio of 16:9, a min resolution of 512x288 and max size of 8MB)
Returns the mod id on a string when upload was sucessful.

### update_mod(mod_id: int, modfile_path: String, user_token: String,)

Needs a token obtained using one of the auth functions
Compresses the file at modfile_path onto a zip file and updates the mod.io mod.
Returns a bool value.

## Auth

These functions returns a String with a token on success or an empty String otherwise

### login_with_steam(ticket: String)

Uses the Steam user auth ticket to login to mod.io

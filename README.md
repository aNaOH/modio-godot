
![Mod.io For Godot](https://github.com/aNaOH/modio-godot/blob/main/logo.svg?raw=true)

Mod.io For Godot using Rust is going to be deprecated, this branch is published for everyone to use and contribute.

The main branch contains the lastest working version of the plugin, the master branch contains the lastest version of the code, even if it not works or if I'm stuck with something.

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
- date_updated
- date_live
- profile_url
- modfile_url
- modfile_name
- modfile_size
- tags

get_mods() needs a string argument for the search query, if you want to list all mods just use "" as the argument

### upload_mod(api_key: String, modfile_path: String, name: String, summary: String, thumbnail_path: String)

Needs an API key obtained using one of the auth functions
Compresses the file at modfile_path onto a zip file and uploads it to mod.io with the specified name and summary, also uploads the thumbnail at thumbnail_path (This image has to have an aspect ratio of 16:9, a min resolution of 512x288 and max size of 8MB)
Returns the mod dictionary.

## Auth

These functions returns a Dictionary with an api key located in dictionary["api_key"]

### login_with_email(email: String, password: String)

Uses email and password to login to mod.io

### login_with_steam(app_id: int, ticket: String)

Uses the Steam AppID and user auth ticket to login to mod.io

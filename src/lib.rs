use godot::prelude::*;
use godot::engine::Node;
use godot::builtin::meta::GodotConvert;

use modio::mods::filters::Tags;
use modio::types::id::GameId;
use modio::{Credentials, Modio};
use modio::filter::prelude::*;
use modio::mods::Mod;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use zip::write::FileOptions;
use zip::ZipWriter;

use reqwest::Client;
use reqwest::multipart;

use tokio::fs::read;

use image::GenericImageView;

struct ModIOAddon;

#[gdextension]
unsafe impl ExtensionLibrary for ModIOAddon {}

pub struct ModIOMod {
    pub id: u64,
    pub name: GString,
    pub submitter: GString,
    pub summary: GString,
    pub description: GString,
    pub date_updated: i64,
    pub date_live: i64,
    pub thumb_url: GString,
    pub profile_url: GString,
    pub modfile_url: GString,
    pub modfile_name: GString,
    pub modfile_size: i64,
    pub tags: PackedStringArray,
}

impl GodotConvert for ModIOMod {
    type Via = Dictionary;
}

impl ToGodot for ModIOMod {


    fn into_godot(self) -> Self::Via {
        let mut dictionary = Dictionary::new();
        dictionary.insert("id", self.id);
        dictionary.insert("name", self.name.clone());
        dictionary.insert("submitter", self.submitter.clone());
        dictionary.insert("summary", self.summary.clone());
        dictionary.insert("description", self.description.clone());
        dictionary.insert("date_updated", self.date_updated);
        dictionary.insert("date_live", self.date_live);
        dictionary.insert("thumb_url", self.thumb_url.clone());
        dictionary.insert("profile_url", self.profile_url.clone());
        dictionary.insert("modfile_url", self.modfile_url.clone());
        dictionary.insert("modfile_name", self.modfile_name.clone());
        dictionary.insert("modfile_size", self.modfile_size.clone());
        dictionary.insert("tags", self.tags.clone());


        dictionary
    }

    fn to_variant(&self) -> Variant {
        let mut dictionary = Dictionary::new();
        dictionary.insert("id", self.id);
        dictionary.insert("name", self.name.clone());
        dictionary.insert("submitter", self.submitter.clone());
        dictionary.insert("summary", self.summary.clone());
        dictionary.insert("description", self.description.clone());
        dictionary.insert("date_updated", self.date_updated);
        dictionary.insert("date_live", self.date_live);
        dictionary.insert("thumb_url", self.thumb_url.clone());
        dictionary.insert("profile_url", self.profile_url.clone());
        dictionary.insert("modfile_url", self.modfile_url.clone());
        dictionary.insert("modfile_name", self.modfile_name.clone());
        dictionary.insert("modfile_size", self.modfile_size.clone());
        dictionary.insert("tags", self.tags.clone());

        Variant::from(dictionary)
    }

    fn to_godot(&self) -> Self::Via {
        let mut dictionary = Dictionary::new();
        dictionary.insert("id", self.id);
        dictionary.insert("name", self.name.clone());
        dictionary.insert("submitter", self.submitter.clone());
        dictionary.insert("summary", self.summary.clone());
        dictionary.insert("description", self.description.clone());
        dictionary.insert("date_updated", self.date_updated);
        dictionary.insert("date_live", self.date_live);
        dictionary.insert("thumb_url", self.thumb_url.clone());
        dictionary.insert("profile_url", self.profile_url.clone());
        dictionary.insert("modfile_url", self.modfile_url.clone());
        dictionary.insert("modfile_name", self.modfile_name.clone());
        dictionary.insert("modfile_size", self.modfile_size.clone());
        dictionary.insert("tags", self.tags.clone());


        dictionary
    }
}

impl ModIOMod {
    fn from_mod(mod_info: &Mod) -> Self {
        let (modfile_url, modfile_name, modfile_size) = if let Some(modfile) = &mod_info.modfile {
            (
                modfile.download.binary_url.as_str(),
                modfile.filename.as_str(),
                modfile.filesize_uncompressed
            )
        } else {
            ("", "", 0)
        };

        let tags: PackedStringArray = mod_info
            .tags
            .iter()
            .map(|tag| tag.name.as_str().into())
            .collect();


        let desc = mod_info.description_plaintext.as_ref().unwrap_or(&"".to_string()).clone();


        Self {
            id: mod_info.id.get(),
            name: mod_info.name.as_str().into(),
            submitter: mod_info.submitted_by.username.as_str().into(),
            summary: mod_info.summary.as_str().into(),
            description: desc.into(),
            date_updated: mod_info.date_updated as i64,
            date_live: mod_info.date_live as i64,
            thumb_url: mod_info.logo.thumb_1280x720.as_str().into_godot(),
            profile_url: mod_info.profile_url.as_str().into(),
            modfile_url: modfile_url.into(),
            modfile_name: modfile_name.into(),
            modfile_size: modfile_size as i64,
            tags,
        }
    }
}

struct ModIOClient {
    client: Modio,
    id: u64
}

impl ModIOClient {
    fn new(api: &String, game: u64) -> Option<Self> {
        match Modio::new(Credentials::new(api)) {
            Ok(modio_instance) => Some(Self { client: modio_instance, id: game }),
            Err(_) => None,
        }
    }

    async fn compress_to_zip(file_path: &str, zip_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(file_path);
        let zip_file = File::create(zip_path)?;
        let mut zip = ZipWriter::new(BufWriter::new(zip_file));

        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().unwrap().to_string_lossy().into_owned();
                    let mut f = File::open(&path)?;
                    zip.start_file(name, FileOptions::default())?;
                    std::io::copy(&mut f, &mut zip)?;
                }
            }
        } else if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let mut f = File::open(&path)?;
            zip.start_file(name, FileOptions::default())?;
            std::io::copy(&mut f, &mut zip)?;
        }

        zip.finish()?;
        Ok(())
    }

    async fn upload_mod_via_api(&self, modfile_path: &str, name: &str, summary: &str, api_key: &str, thumbnail_path: &str) -> Result<ModIOMod, Box<dyn std::error::Error>> {
        let zip_path = format!("{}.zip", modfile_path);
        Self::compress_to_zip(modfile_path, &zip_path).await?;

        let modfile = read(&zip_path).await?;
        let client = Client::new();

        let mut form = multipart::Form::new()
            .text("name", name.to_string())
            .text("summary", summary.to_string())
            .part("modfile", multipart::Part::bytes(modfile).file_name("mod.zip"));

            let thumbnail = read(thumbnail_path).await?;
            let img = image::open(thumbnail_path)?;

            if img.width() * 9 != img.height() * 16 || img.width() < 512 || img.height() < 288 {
                return Err("Thumbnail must be 16:9 and at least 512x288".into());
            }

            if thumbnail.len() > 8 * 1024 * 1024 {
                return Err("Thumbnail must be less than 8MB".into());
            }

            form = form.part("logo", multipart::Part::bytes(thumbnail).file_name("thumbnail.png"));

        let response = client.post(format!("https://api.mod.io/v1/games/{}/mods", self.id))
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .await?
            .json::<Mod>()
            .await?;

        let mod_info = ModIOMod::from_mod(&response);
        Ok(mod_info)
    }

    pub async fn login_with_steam(&self, ticket: &str) -> Result<String, Box<dyn std::error::Error>> {

        let auth = self.client.auth().external(modio::auth::SteamOptions::new(ticket)).await?;

        if !auth.api_key.is_empty() {
            Ok(auth.api_key)
        } else {
            Err("Failed to login with Steam".into())
        }
    }
}

#[derive(GodotClass)]
#[class(base = Node)]
struct ModIO {
    client: Option<ModIOClient>,
}

#[godot_api]
impl INode for ModIO {
    fn init(_node: Base<Node>) -> Self {
        Self { client: None }
    }
}

#[godot_api]
impl ModIO {
    #[func]
    fn connect(&mut self, api_key: GString, game: u64) -> bool {
        if self.client.is_none() {
            if let Some(client) = ModIOClient::new(&api_key.to_string(), game) {
                self.client = Some(client);
                godot_print!("Mod.io Client connected");
            } else {
                godot_print!("Failed to connect Mod.io Client");
            }
        }

        self.client.is_some()
    }

    async fn get_mods_async_inner(&self, query: GString, tags: GString, per_page: usize, page: usize) -> Option<Array<Dictionary>> {
        if let Some(ref client) = self.client {
            // Example: Get mods (replace with your actual parameters)
            let mut f = Filter::default().and(with_limit(per_page).offset(per_page*page));

            if query != "".into() {
                f = Fulltext::eq(query).and(Tags::_in(tags));
            }

            match client
                .client
                .game(GameId::new(client.id))
                .mods()
                .search(f)
                .collect()
                .await
            {
                Ok(mods) => {
                    let mut mod_vec = Array::new();
                    for m in mods {

                        mod_vec.insert(mod_vec.len(), ModIOMod::from_mod(&m).to_godot())
                    }

                    Some(mod_vec)
                },
                Err(err) => {
                    // Imprimir el error y devolver None
                    godot_print!("Error getting mods: {:?}", err);
                    None
                }
            }
        } else {
            None
        }
    }

    
    // Función #[func] que invoca la función asíncrona intermedia
    #[func]
    fn get_mods(&self, query: GString, page: u16, per_page: u16) -> Array<Dictionary> {

        // Crear una nueva tarea y ejecutarla
        let result = async {
            match self.get_mods_async_inner(query, "".into(), per_page.into(), page.into()).await {
                Some(mods) => {
                    // Imprimir información sobre los mods
                    godot_print!("Mods found");
                    // Devolver los mods
                    mods
                }
                None => {
                    // Imprimir mensaje de error y devolver un vector vacío
                    godot_print!("Error getting mods or Mod.io Client not connected");
                    Array::new()
                }
            }
        };
    
        // Crear una nueva runtime de tokio y ejecutar la tarea
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mods = rt.block_on(result);
    
        // Devolver el resultado de la tarea
        mods
    }

    #[func]
    fn upload_mod(&self, api_key: GString, modfile_path: GString, name: GString, summary: GString, thumbnail_path: GString) -> Dictionary {
        let empty_dict = Dictionary::new();
        if let Some(ref client) = self.client {
            // Create a new task and execute it
            let result = async {

                match client.upload_mod_via_api(&modfile_path.to_string(), &name.to_string(), &summary.to_string(), &api_key.to_string(), &thumbnail_path.to_string()).await {
                    Ok(mod_info) => {
                        // Print information about the uploaded mod
                        godot_print!("Mod uploaded successfully: {}", mod_info.name);
                        // Return the uploaded mod
                        mod_info.to_godot()
                    }
                    Err(err) => {
                        // Print error message and return an empty dictionary
                        godot_print!("Error uploading mod: {:?}", err);
                        empty_dict
                    }
                }
            };

            // Create a new tokio runtime and execute the task
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(result)
        } else {
            empty_dict
        }
    }

    #[func]
    fn login_with_steam(&self, ticket: GString) -> GString {
        if let Some(ref client) = self.client {
            let result = async {
                match client.login_with_steam(&ticket.to_string()).await {
                    Ok(api_key) => {
                        api_key.to_godot()
                    }
                    Err(err) => {
                        godot_print!("Error logging in with Steam: {:?}", err);
                        "".to_godot()
                    }
                }
            };

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(result)
        } else {
            "".to_godot()
        }
    }
}
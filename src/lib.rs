use godot::prelude::*;
use godot::engine::Node;
use godot::builtin::meta::GodotConvert;

use modio::files::AddFileOptions;
use modio::mods::filters::Tags;
use modio::types::id::{GameId, Id};
use modio::{Credentials, Modio};
use modio::filter::prelude::*;
use modio::mods::{AddModOptions, Mod};

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use zip::write::FileOptions;
use zip::ZipWriter;

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
    id: u64,
    game_api : String
}

impl ModIOClient {
    fn new(api: &String, game: u64) -> Option<Self> {
        match Modio::new(Credentials::new(api)) {
            Ok(modio_instance) => Some(Self { client: modio_instance, id: game, game_api: api.to_string() }),
            Err(_) => None,
        }
    }

    async fn search_mods(&self, query: &str, tags: &Vec<&str>, page: usize, per_page: usize) -> Result<Vec<Mod>, Box<dyn std::error::Error>> {
        if page < 1 {
            return Err("Page must start on 1".into());
        }

        // Crear el filtro básico con paginación
        let mut f = Filter::default().and(with_limit(per_page).offset(per_page * (page - 1)));

        // Agregar búsqueda por texto si el query no está vacío
        if !query.is_empty() {
            f = f.and(Fulltext::eq(query));
        }

        // Agregar búsqueda por etiquetas si hay etiquetas especificadas
        if !tags.is_empty() {
            for tag in tags {
                f = f.and(Tags::eq(*tag));
            }
        }

        // Realizar la búsqueda con el filtro aplicado
        let mods = self.client.game(GameId::new(self.id)).mods().search(f).collect().await?;

        Ok(mods)
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

    async fn upload_mod_via_api(&self, modfile_path: &str, name: &str, summary: &str, user_token: &str, thumbnail_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let zip_path = modfile_path.to_owned() + ".zip";
        Self::compress_to_zip(modfile_path, &zip_path).await?;


        let thumbnail = read(thumbnail_path).await?;
        let img = image::open(thumbnail_path)?;

        if img.width() * 9 != img.height() * 16 || img.width() < 512 || img.height() < 288 {
            return Err("Thumbnail must be 16:9 and at least 512x288".into());
        }

        if thumbnail.len() > 8 * 1024 * 1024 {
            return Err("Thumbnail must be less than 8MB".into());
        }

        let user_client = self.client.with_credentials(Credentials::with_token(self.game_api.as_str(), user_token));

        let new_mod = user_client.game(GameId::new(self.id)).mods().add(AddModOptions::new(name, thumbnail_path, summary)).await?;

        user_client.game(GameId::new(self.id)).mod_(new_mod.id).files().add(AddFileOptions::with_file(zip_path)).await?;

        Ok(new_mod.id.to_string())
    }

    pub async fn login_with_steam(&self, ticket: &str) -> Result<String, Box<dyn std::error::Error>> {

        let auth = self.client.auth().external(modio::auth::SteamOptions::new(ticket)).await?;

        if auth.token.is_some() {
            let token = auth.token.ok_or("Token not found")?;
            Ok(token.value)
        } else {
            Err("Failed to login with Steam".into())
        }
    }

    async fn update_mod(&self, mod_id: u64, modfile_path: &str, user_token: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let user_client = self.client.with_credentials(Credentials::with_token(self.game_api.as_str(), user_token));

        let mod_ref = user_client.game(GameId::new(self.id)).mod_(Id::new(mod_id));

        let version = mod_ref.files().search(Filter::default()).collect().await?.len();
        
        let zip_path = modfile_path.to_owned() + version.to_string().as_str() + ".zip";
        Self::compress_to_zip(modfile_path, &zip_path).await?;

        mod_ref.files().add(AddFileOptions::with_file(zip_path)).await?;

        Ok(true)
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

    // Función #[func] que invoca la función asíncrona intermedia
    #[func]
    fn get_mods(&self, query: GString, page: u16, per_page: u16, tags: PackedStringArray) -> Array<Dictionary> {
        if let Some(ref client) = self.client {
            // Crear una nueva tarea y ejecutarla
            let result = async {
                // Convertir PackedStringArray a Vec<String>
                let tags_array: Vec<String> = tags.to_vec().into_iter().map(|tag| tag.to_string()).collect();

                // Convertir Vec<String> a Vec<&str>
                let tags_str_array: Vec<&str> = tags_array.iter().map(|s| s.as_str()).collect();

                match client.search_mods(query.to_string().as_str(), &tags_str_array, page.into(), per_page.into()).await {
                    Ok(mod_list) => {
                        let mut array = Array::new();
                        
                        for mod_info in mod_list {
                            array.push(ModIOMod::from_mod(&mod_info).to_godot());
                        }

                        array
                    }
                    Err(err) => {
                        godot_error!("Error searching mods! {:?}", err);
                        Array::new()
                    }
                }
            };

            // Crear un nuevo tokio runtime y ejecutar la tarea
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(result)
        } else {
            Array::new()
        }
    }


    #[func]
    fn upload_mod(&self, user_token: GString, modfile_path: GString, name: GString, summary: GString, thumbnail_path: GString) -> GString {
        if let Some(ref client) = self.client {
            // Create a new task and execute it
            let result = async {

                match client.upload_mod_via_api(&modfile_path.to_string(), &name.to_string(), &summary.to_string(), &user_token.to_string(), &thumbnail_path.to_string()).await {
                    Ok(mod_info) => {
                        // Print information about the uploaded mod
                        godot_print!("Mod uploaded successfully");
                        // Return the uploaded mod
                        mod_info.to_godot()
                    }
                    Err(err) => {
                        // Print error message and return an empty dictionary
                        godot_print!("Error uploading mod: {:?}", err);
                        "".to_godot()
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
            "".to_godot()
        }
    }

    #[func]
    fn update_mod(&self, mod_id: u64, modfile_path: GString, user_token: GString) -> bool {
        if let Some(ref client) = self.client {
            // Create a new task and execute it
            let result = async {

                match client.update_mod(mod_id, &modfile_path.to_string(), &user_token.to_string()).await {
                    Ok(mod_info) => {
                        // Print information about the uploaded mod
                        godot_print!("Mod uploaded successfully");
                        // Return the uploaded mod
                        mod_info.to_godot()
                    }
                    Err(err) => {
                        // Print error message and return an empty dictionary
                        godot_print!("Error uploading mod: {:?}", err);
                        false
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
            false
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
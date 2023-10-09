use godot::prelude::*;
use godot::engine::Node;
use godot::engine::NodeVirtual;


use godot::prelude::meta::VariantMetadata;
use modio::{Credentials, Modio};
use modio::filter::prelude::*;
use modio::mods::Mod;

struct ModIOAddon;

#[gdextension]
unsafe impl ExtensionLibrary for ModIOAddon {}

struct ModIOClient {
    client: Modio,
    id: u32
}

impl ModIOClient {
    fn new(api: &String, game: u32) -> Option<Self> {
        match Modio::new(Credentials::new(api)) {
            Ok(modio_instance) => Some(Self { client: modio_instance, id: game }),
            Err(_) => None,
        }
    }
}

#[derive(GodotClass)]
#[class(base = Node)]
struct ModIO {
    client: Option<ModIOClient>,
}

#[godot_api]
impl NodeVirtual for ModIO {
    fn init(_node: Base<Node>) -> Self {
        godot_print!("Hello, world!");

        Self { client: None }
    }
}

struct ModIOMod {
    pub id: u32,
    pub date_updated: i64,
    pub date_live: i64,
    pub profile_url: GodotString,
    pub modfile_url: GodotString,
    pub modfile_name: GodotString,
    pub modfile_size: i64,
    pub tags: PackedStringArray,
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

        Self {
            id: mod_info.id,
            date_updated: mod_info.date_updated as i64,
            date_live: mod_info.date_live as i64,
            profile_url: mod_info.profile_url.as_str().into(),
            modfile_url: modfile_url.into(),
            modfile_name: modfile_name.into(),
            modfile_size: modfile_size as i64,
            tags,
        }
    }

    
}

impl ToVariant for ModIOMod {
    fn to_variant(&self) -> Variant {
        let mut dictionary = Dictionary::new();
        dictionary.insert("id", self.id);
        dictionary.insert("date_updated", self.date_updated);
        dictionary.insert("date_live", self.date_live);
        dictionary.insert("profile_url", self.profile_url.clone());
        dictionary.insert("modfile_url", self.modfile_url.clone());
        dictionary.insert("modfile_name", self.modfile_name.clone());
        dictionary.insert("modfile_size", self.modfile_size.clone());
        dictionary.insert("tags", self.tags.clone());


        Variant::from(dictionary)
    }
}


impl VariantMetadata for ModIOMod {
    fn variant_type() -> VariantType {
        VariantType::Dictionary
    }
}

#[godot_api]
impl ModIO {
    #[func]
    fn connect(&mut self, api_key: GodotString, game: u32) -> bool {
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

    async fn get_mods_async_inner(&self, query: GodotString) -> Option<Array<ModIOMod>> {
        if let Some(ref client) = self.client {
            // Example: Get mods (replace with your actual parameters)
            let mut f = Filter::default();

            if query != "".into() {
                f = Fulltext::eq(query);
            }

            match client
                .client
                .game(client.id)
                .mods()
                .search(f)
                .collect()
                .await
            {
                Ok(mods) => {
                    let mut mod_vec = Array::new();
                    for m in mods {

                        mod_vec.insert(mod_vec.len(), ModIOMod::from_mod(&m))
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
    fn get_mods(&self, query: GodotString) -> Array<ModIOMod> {
        // Crear una nueva tarea y ejecutarla
        let result = async {
            match self.get_mods_async_inner(query).await {
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
}
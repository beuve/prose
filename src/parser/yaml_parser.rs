use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use bimap::BiMap;
use serde::Deserialize;

use crate::{
    engine::{actors::AMActor, scheduler::Scheduler},
    parser::actors_parser::ACTORS,
};
pub type Result<T> = std::result::Result<T, serde_yaml::Error>;

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub time_window: usize,
    pub dt: f64,
}

#[derive(Debug, Deserialize)]
struct ActorConfig {
    #[serde(rename = "type")]
    pub actor_type: String,

    #[serde(default)]
    pub source: bool,

    #[serde(flatten, rename = "config", default)]
    pub custom_params: serde_yaml::Value,

    #[serde(default)]
    pub clients: HashMap<String, HashMap<String, u32>>,
}

#[derive(Debug, Deserialize)]
struct ParsedConfig {
    global: GlobalConfig,
    components: Vec<String>,
    actors: HashMap<String, ActorConfig>,
}

pub struct Config {
    pub global: GlobalConfig,
    pub actors: HashMap<String, AMActor>,
    pub init_sources: Vec<String>,
    pub scheduler: Scheduler,
    pub loging_config: LogingConfig,
}

pub struct LogingConfig {
    pub actors_indices: BiMap<u16, usize>,
    pub components_codes: BiMap<String, u16>,
    pub actors_codes: BiMap<String, u16>,
    pub offset_id_actors: u16,
}

impl LogingConfig {
    pub fn name_from_code(&self, code: u16) -> String {
        let mut actor_code = code;
        for _ in 0..self.offset_id_actors.ilog10() {
            actor_code /= 10;
        }
        actor_code *= self.offset_id_actors;
        let component_code = code - actor_code;
        format!(
            "{}/{}",
            self.components_codes.get_by_right(&component_code).unwrap(),
            self.actors_codes.get_by_right(&actor_code).unwrap(),
        )
    }

    pub fn num_elements(&self) -> usize {
        self.actors_codes.len()
    }
}

fn connect_to_clients(
    config: &HashMap<String, ActorConfig>,
    actors: &HashMap<String, AMActor>,
    components: &BiMap<String, u16>,
) {
    for (actor_name, actor_config) in config {
        let actor = actors.get(actor_name).unwrap();
        let products = &actor_config.clients;
        for (product_label, clients) in products {
            for (client_name, value) in clients {
                let client = actors.get(client_name).unwrap();
                let code_client = client.lock().unwrap().code();
                actor.lock().unwrap().register(
                    code_client,
                    *components.get_by_left(product_label).unwrap(),
                    *value,
                    client.clone(),
                );
            }
        }
    }
}

fn parse_actors(
    config: &ParsedConfig,
    components: &BiMap<String, u16>,
    scheduler: &Scheduler,
    offset_id_actors: u16,
) -> (HashMap<String, AMActor>, BiMap<String, u16>) {
    let mut actors: HashMap<String, AMActor> = HashMap::new();
    let mut actors_codes: BiMap<String, u16> = BiMap::new();
    let mut index = offset_id_actors;
    let actors_config = &config.actors;
    for (actor_name, actor_config) in actors_config {
        let actors_callbacks = ACTORS.lock().unwrap();
        let actor_callback = actors_callbacks.get(&actor_config.actor_type).expect("");
        actors_codes.insert(actor_name.clone(), index);
        let actor = actor_callback(
            &actor_config.custom_params,
            index,
            components,
            scheduler.clone(),
            config.global.dt,
        )
        .expect("msg");
        actors.insert(actor_name.clone(), actor);
        index += offset_id_actors;
    }
    connect_to_clients(actors_config, &actors, components);
    (actors, actors_codes)
}

fn get_sources(config: &ParsedConfig) -> Vec<String> {
    config
        .actors
        .iter()
        .filter(|a| a.1.source)
        .map(|a| a.0.clone())
        .collect()
}

fn get_actors_indices(
    config: &HashMap<String, ActorConfig>,
    components_codes: &BiMap<String, u16>,
    actors_codes: &BiMap<String, u16>,
) -> BiMap<u16, usize> {
    let mut res = BiMap::new();
    for actor_config in config.values() {
        let clients = &actor_config.clients;
        for (product_label, clients) in clients {
            let product_code = components_codes.get_by_left(product_label).unwrap();
            for client_name in clients.keys() {
                let actor_code = actors_codes.get_by_left(client_name).unwrap();
                let code = actor_code + product_code;
                if !res.contains_left(&code) {
                    res.insert(code, res.len());
                }
            }
        }
    }
    res
}

pub fn parse_config(path: &Path) -> Config {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(&file);
    let scheduler = Scheduler::new();
    let parsed_config: ParsedConfig = serde_yaml::from_reader(reader).unwrap();
    let mut components_codes: BiMap<String, u16> = BiMap::new();
    for (i, c) in parsed_config.components.iter().enumerate() {
        components_codes.insert(c.clone(), (i + 1) as u16);
    }
    let offset_id_actors = ((components_codes.len().ilog10() + 1) * 10) as u16;
    let (actors, actors_codes) = parse_actors(
        &parsed_config,
        &components_codes,
        &scheduler,
        offset_id_actors,
    );
    let actors_indices =
        get_actors_indices(&parsed_config.actors, &components_codes, &actors_codes);
    let init_sources = get_sources(&parsed_config);
    let loging_config = LogingConfig {
        actors_codes,
        actors_indices,
        components_codes,
        offset_id_actors,
    };
    Config {
        global: parsed_config.global,
        actors,
        init_sources,
        scheduler,
        loging_config,
    }
}

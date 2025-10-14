use std::collections::HashMap;
use std::fmt;
use std::fs;

use threadpool::ThreadPool;
use yaml_rust2::yaml::Hash;
use yaml_rust2::{Yaml, YamlLoader};

use crate::analyzer::Sampler;
use crate::engine::actor::AMActor;
use crate::parser::actors_parser::ACTORS;

use super::time_distribution_parser::TIME_CALLBACK;
pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone)]
pub enum ParseError {
    FileNotFound,
    SectionMissing(String),
    SectionWrongType(String),
    UnknownComponent(String),
    UnknownActor(String),
    UnknownTimeDistribution(String),
    WrongFormat(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::FileNotFound => write!(f, "Config file not found"),
            ParseError::SectionMissing(s) => write!(f, "Could not find {} section", s),
            ParseError::SectionWrongType(s) => write!(f, "Section {} has wrong type", s),
            ParseError::UnknownComponent(s) => write!(f, "Unknown component {}", s),
            ParseError::UnknownActor(s) => write!(f, "Unknown actor {}", s),
            ParseError::UnknownTimeDistribution(s) => write!(f, "Unknown time distribution {}", s),
            ParseError::WrongFormat(s) => write!(f, "The config file is not well formatted: {}", s),
        }
    }
}

impl std::convert::From<std::io::Error> for ParseError {
    fn from(_: std::io::Error) -> Self {
        ParseError::FileNotFound
    }
}

pub struct GlobalConfig {
    pub time_window: usize,
    pub dt: f64,
}

pub trait YamlParser {
    fn get(&self, label: &str) -> Result<&Self>;
    fn str(&self) -> Result<&str>;
    fn int(&self) -> Result<usize>;
    fn float(&self) -> Result<f64>;
    fn bool(&self) -> Result<bool>;
    fn hash(&self) -> Result<&Hash>;
}

impl YamlParser for Yaml {
    fn get<'a>(&'a self, label: &str) -> Result<&'a Self> {
        if self[label].is_badvalue() {
            return Err(ParseError::SectionMissing(String::from(label)));
        }
        Ok(&self[label])
    }

    fn str(&self) -> Result<&str> {
        match self.as_str() {
            None => Err(ParseError::SectionWrongType(String::from("unknown"))),
            Some(data) => Ok(data),
        }
    }

    fn int(&self) -> Result<usize> {
        match self.as_i64() {
            None => Err(ParseError::SectionWrongType(String::from("unknown"))),
            Some(data) => Ok(data as usize),
        }
    }

    fn bool(&self) -> Result<bool> {
        match self.as_bool() {
            None => Err(ParseError::SectionWrongType(String::from("unknown"))),
            Some(data) => Ok(data),
        }
    }

    fn hash(&self) -> Result<&Hash> {
        match self.as_hash() {
            None => Err(ParseError::SectionWrongType(String::from("unknown"))),
            Some(data) => Ok(data),
        }
    }

    fn float(&self) -> Result<f64> {
        match self.as_f64() {
            None => Err(ParseError::SectionWrongType(String::from("unknown"))),
            Some(data) => Ok(data),
        }
    }
}

fn parse_global(doc: &Yaml) -> Result<GlobalConfig> {
    if doc.is_badvalue() {
        return Err(ParseError::SectionMissing(String::from("Global")));
    }
    let time_window = doc.get("time_window")?.int()?;
    let dt = doc.get("dt")?.float()?;
    Ok(GlobalConfig { time_window, dt })
}

fn parse_components(doc: &Yaml) -> Result<HashMap<String, u16>> {
    let mut components = HashMap::new();
    let mut id = 1u16;
    for label in doc.clone() {
        let _ = match label.as_str() {
            None => return Err(ParseError::SectionWrongType(String::from("components"))),
            Some(l) => components.insert(String::from(l), id),
        };
        id += 1;
    }
    Ok(components)
}

fn parse_clients(
    doc: &Yaml,
    actors: &mut HashMap<String, AMActor>,
    components: &HashMap<String, u16>,
) -> Result<()> {
    let actors_doc = doc.hash()?;
    for (actor_label, content) in actors_doc {
        let clients = &content["clients"];
        if clients.is_badvalue() {
            continue;
        }
        let actor_label = actor_label.str()?.to_string();
        let actor = actors.get(&actor_label).unwrap();
        for (client_label, products) in clients.hash()? {
            let client = actors.get(&client_label.str()?.to_string()).unwrap();
            let client_code = client.lock().unwrap().code();
            for (product_label, value) in products.hash()? {
                actor.lock().unwrap().register(
                    client_code,
                    *components.get(&product_label.str()?.to_string()).unwrap(),
                    value.int()? as u32,
                    client.clone(),
                );
            }
        }
    }
    Ok(())
}

fn parse_actors(
    doc: &Yaml,
    components: &HashMap<String, u16>,
    threadpool: ThreadPool,
) -> Result<HashMap<String, AMActor>> {
    let actors = doc.hash()?;
    let mut res: HashMap<String, AMActor> = HashMap::new();
    let index_step = ((components.len().ilog10() + 1) * 10) as u16;
    let mut index = index_step;
    for (label, content) in actors {
        let label = label.str()?.to_string();
        let actor_type = content.get("type")?.str()?.to_string();
        let actors = ACTORS.lock().unwrap();
        let actor_callback = actors
            .get(&actor_type)
            .ok_or(ParseError::UnknownActor(actor_type))?;
        res.insert(
            label,
            actor_callback(content, index, components.clone(), threadpool.clone())?,
        );
        index += index_step;
    }
    Ok(res)
}

pub struct ActorLogInfos {
    pub index: usize,
    pub product_code: String,
    pub time_sampler: Option<Sampler>,
}

fn parse_logs(
    doc: &Yaml,
    components: &HashMap<String, u16>,
    actors: &HashMap<String, AMActor>,
    dt: f64,
) -> Result<HashMap<u16, ActorLogInfos>> {
    let actors_doc = doc.hash()?;
    let mut res: HashMap<u16, ActorLogInfos> = HashMap::new();
    for (actor_label, content) in actors_doc {
        let actor_label = actor_label.str()?.to_string();
        let actor = actors.get(&actor_label).unwrap();
        let log = &content["log"];
        if log.is_badvalue() {
            continue;
        }
        for (product_label, content) in log.hash()? {
            let product_label = product_label.str()?.to_string();
            let code = *components.get(&product_label).unwrap() + actor.lock().unwrap().code();

            if content.is_null() {
                res.insert(
                    code,
                    ActorLogInfos {
                        product_code: format!("{actor_label}/{product_label}"),
                        index: res.len(),
                        time_sampler: None,
                    },
                );
                continue;
            }
            let content = content.hash()?;
            if content.len() > 1 {
                return Err(ParseError::WrongFormat(format!(
                    "Multiple distributions were provided in actor {}",
                    actor_label
                )));
            }
            let callback_doc = content.keys().next().unwrap();
            let callback_name = callback_doc.str()?.to_string();
            let time_callbacks = TIME_CALLBACK.lock().unwrap();
            let time_creation_callback = time_callbacks.get(&callback_name).unwrap();
            let time_callback = time_creation_callback(content.get(callback_doc).unwrap(), dt)?;
            res.insert(
                code,
                ActorLogInfos {
                    product_code: format!("{actor_label}/{product_label}"),
                    index: res.len(),
                    time_sampler: Some(time_callback),
                },
            );
        }
    }
    Ok(res)
}

fn parse_init_sources(doc: &Yaml) -> Result<Vec<String>> {
    let mut res = vec![];
    let actors_doc = doc.hash()?;
    for (actor_label, content) in actors_doc {
        let actor_label = actor_label.str()?.to_string();
        let init_doc = &content["source"];
        if init_doc.is_badvalue() || !init_doc.bool()? {
            continue;
        }
        res.push(actor_label);
    }
    Ok(res)
}

pub struct Config {
    pub global: GlobalConfig,
    pub actors: HashMap<String, AMActor>,
    pub components: HashMap<String, u16>,
    pub logs: HashMap<u16, ActorLogInfos>,
    pub init_sources: Vec<String>,
    pub pool: ThreadPool,
}

pub fn parse_config(path: String, pool: ThreadPool) -> Result<Config> {
    let config: String = fs::read_to_string(path)?;
    let docs = YamlLoader::load_from_str(&config).unwrap();
    let doc = &docs[0];
    let global_doc = doc.get("global")?;
    let global = parse_global(global_doc)?;
    let components = doc.get("components")?;
    let components = parse_components(components)?;
    let actors_doc = doc.get("actors")?;
    let mut actors = parse_actors(actors_doc, &components, pool.clone())?;
    parse_clients(actors_doc, &mut actors, &components)?;
    let logs = parse_logs(actors_doc, &components, &actors, global.dt)?;
    let init_sources = parse_init_sources(actors_doc)?;
    Ok(Config {
        global,
        actors,
        components,
        logs,
        init_sources,
        pool,
    })
}

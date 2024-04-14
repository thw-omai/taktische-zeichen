use std::fs;

use serde::Deserialize;

pub fn parse() -> (Config, HelferConfig) {
    let config_text = fs::read_to_string("config.toml").expect("Couldn't parse config file.");
    let helfer_text = fs::read_to_string("helfer.toml").expect("Couldn't parse helfer file.");

    let config: Config = toml::from_str(&*config_text).unwrap();
    let helfer_config: HelferConfig = toml::from_str(&*helfer_text).unwrap();

    return (config, helfer_config);
}

#[derive(Debug, Deserialize,Clone)]
pub(crate) struct Config {
    pub(crate) enable_png: bool,
    pub(crate) thw: Vec<DescriptionObjects>,
    pub(crate) fw: Vec<DescriptionObjects>,
    pub(crate) zoll: Vec<DescriptionObjects>,
    pub(crate) rettung: Vec<DescriptionObjects>,
    pub(crate) pol: Vec<DescriptionObjects>,
    pub(crate) bw: Vec<DescriptionObjects>,
    pub(crate) kats: Vec<DescriptionObjects>,
    pub(crate) alle: Vec<DescriptionObjects>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct DescriptionObjects {
    pub(crate) template: String,
    pub(crate) zug: String,
    pub(crate) names: String,
    pub(crate) special: String,
    pub(crate) dir: String,
}


#[derive(Debug, Deserialize, Clone)]
pub(crate) struct HelferConfig {
    pub(crate) enabled: bool,
    pub(crate) personen: Option<Vec<Person>>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Person {
    pub(crate) helfer: String,
    pub(crate) organisation: String,
    pub(crate) zug: String,
    pub(crate) template: String,
    pub(crate) value: String,
}
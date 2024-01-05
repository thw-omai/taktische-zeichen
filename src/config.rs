use std::fs;
use serde::Deserialize;

use toml::Value;

pub fn parse() -> (Vec<ParseableObject>, ThwConfig) {
    let mut objects: Vec<ParseableObject> = vec![];
    let config_text = fs::read_to_string("config.toml").expect("Couldn't parse config file.");

    let value = config_text.parse::<Value>().unwrap();
    parse_organisation(&mut objects, &value);

    let config: ThwConfig = toml::from_str(&*config_text).unwrap();

    return (objects, config);
}

fn parse_organisation(objects: &mut Vec<ParseableObject>, value: &Value) {
    match value {
        Value::Table(table) => {
            for (organisation, v) in table.iter() {
                parse_organisation2(objects, v, organisation);
            }
        }
        _ => {}
    }
}

fn parse_organisation2(objects: &mut Vec<ParseableObject>, value: &Value, organisation: &String) {
    match value {
        Value::Table(table) => {
            for (zug, v) in table.iter() {
                parse_config2(objects, v, organisation, zug);
            }
        }
        _ => {}
    }
}

fn parse_config2(objects: &mut Vec<ParseableObject>, value: &Value, organisation: &String, zug: &String) {
    match value {
        Value::Array(array) => {
            for v in array.iter() {
                parse_config(objects, v, organisation, zug);
            }
        }
        _ => {}
    }
}

fn parse_config(objects: &mut Vec<ParseableObject>, value: &Value, organisation: &String, zug: &String) {
    match value {
        Value::Table(table) => {
            for (type_object, v) in table.iter() {
                parse_config_string(objects, v, organisation, zug, type_object);
            }
        }
        _ => {}
    }
}

fn parse_config_string(objects: &mut Vec<ParseableObject>, value: &Value, organisation: &String, zug: &String, type_object: &String) {
    match value {
        Value::String(string) => {
            objects.push(ParseableObject {
                organisation: organisation.to_string(),
                zug: zug.to_string(),
                type_object: type_object.to_string(),
                value: string.split(",").map(|s| s.to_string()).collect(),
            })
        }
        _ => {}
    }
}

#[derive(Debug)]
pub struct ParseableObject {
    pub(crate) organisation: String,
    pub(crate) zug: String,
    pub(crate) type_object: String,
    pub(crate) value: Vec<String>,
}


#[derive(Debug, Deserialize)]
pub(crate) struct ThwConfig {
    pub(crate) thw: ThwEntry,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ThwEntry {
    pub(crate) alle:Vec<ThwAlle>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ThwAlle {
    pub(crate) orte: String,
    pub(crate) helfer_namen: String,
}
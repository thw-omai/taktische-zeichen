use std::{
    collections::HashMap,
    fs::{
        self,
        File,
    },
    io::{
        self,
        Read,
    },
    path::{Component, Path,PathBuf},
};

use base64::{
    Engine,
    engine::general_purpose,
};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use walkdir::WalkDir;

use crate::config::DescriptionObjects;

mod svg_tools;
mod config;

fn main() {
    let (cfg, helfer_config) = config::parse();

    let mut tera = match Tera::new("icons/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.autoescape_on(vec![".template.svg"]);

    cfg.thw.iter().for_each(|current| {
        generate_svg("THW", &mut tera, &current);
    });
    cfg.fw.iter().for_each(|current| {
        generate_svg("FW", &mut tera, &current);
    });
    cfg.pol.iter().for_each(|current| {
        generate_svg("POL", &mut tera, &current);
    });
    cfg.zoll.iter().for_each(|current| {
        generate_svg("Zoll", &mut tera, &current);
    });
    cfg.bw.iter().for_each(|current| {
        generate_svg("BW", &mut tera, &current);
    });
    cfg.rettung.iter().for_each(|current| {
        generate_svg("Rettung", &mut tera, &current);
    });
    cfg.kats.iter().for_each(|current| {
        generate_svg("KatS", &mut tera, &current);
    });
    cfg.alle.iter().for_each(|current| {
        generate_svg("Alle", &mut tera, &current);
    });

    let thw_config = &helfer_config
        .personen
        .unwrap_or(vec! {});
    if helfer_config.enabled {
        thw_config.iter().for_each(|person| {
            vec![true, false].iter().for_each(|inverted| {
                person.helfer.split(",").for_each(|helfer| {
                    person.value.split(",").for_each(|val| {
                        let target_file_path = format!(
                            "build/custom/svg/{}/{}/{}/{}-{}-{}.svg",
                            if *inverted { "inverted" } else { "original" },
                            &person.organisation,
                            &person.zug,
                            &helfer,
                            person.template,
                            val,
                        );
                        let filename = format!(
                            "{}/{}/{}.template.svg",
                            "icons",
                            &person.organisation,
                            val
                        );

                        process_file_common(
                            &filename,
                            &target_file_path,
                            &*person.organisation,
                            &person.template,
                            *inverted,
                            &*val,
                            "",
                            "",
                            helfer,
                            "personen",
                            tera.clone(),
                        )
                    });
                });
            });
        });
    }
    if cfg.enable_png {
        svg_tools::convert_svg()
    }
    copy_static();

    create_drawio()
}

#[derive(Serialize, Deserialize, Clone)]
struct DrawIoLibEntry {
    data: String,
    w: i32,
    h: i32,
    title: String,
    aspect: String,
}

fn create_drawio() {
    let mut data: HashMap<String, Vec<DrawIoLibEntry>> = HashMap::new();
    process_entries("build", |path: PathBuf| {
        let entry = DrawIoLibEntry {
            data: format!(
                "data:image/svg+xml;base64,{}",
                file_to_base64(&path.to_str().unwrap()).unwrap()
            ),
            w: 256,
            h: 256,
            title: path_to_title("build", path.clone()),
            aspect: "fixed".to_string(),
        };
        let map_id = path_to_id("build", path.parent().unwrap());
        let mut vec = match data.get_mut(map_id.as_str()) {
            None => Vec::new(),
            Some(item) => item.to_vec()
        };
        vec.push(entry);
        data.insert(map_id, vec);
    });

    fs::create_dir_all("build/drawio")
        .expect("Couldn't create drawio directory");

    data.iter().for_each(|(key, item)| {
        let json_string = serde_json::to_string(item)
            .expect("Failed to serialize to JSON");

        println!("Save to {}", format!("build/{}", key).as_str());
        save_to_file(
            format!("build/drawio/{}.xml", key).as_str(),
            format!("<mxlibrary>{}</mxlibrary>", &json_string).as_str(),
        )
    });
}

fn path_to_title(
    match_name: &str,
    path: PathBuf,
) -> String {
    let mut result = String::new();

    for component in path.components() {
        if let Component::Normal(name) = component {
            if name == match_name {
                result.clear();
            } else if name != "svg" {
                result.push_str(
                    name.to_str()
                        .unwrap_or("")
                        .replace("-", " ")
                        .as_str()
                );
                result.push(' ');
            }
        }
    }

    result
        .trim()
        .trim_end_matches(".svg")
        .to_string()
}

fn path_to_id(
    match_name: &str,
    path: &Path,
) -> String {
    let mut result = String::new();

    for component in path.components() {
        if let Component::Normal(name) = component {
            if name == match_name {
                result.clear();
            } else if name != "svg" {
                result.push_str(name.to_str().unwrap_or(""));
                result.push('-');
            }
        }
    }

    let mut result = result.trim_end_matches("-").to_string();
    if result.contains("original") {
        result = result.trim_start_matches("original-").to_string();
        result = format!("{}-original", result);
    } else if result.contains("inverted") {
        result = result.trim_start_matches("inverted-").to_string();
        result = format!("{}-inverted", result);
    }
    result
}

fn process_entries<F>(
    directory: &str,
    process_fn: F,
) where
    F: FnMut(PathBuf),
{
    WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            return if let Some(extension) = entry.path().extension() {
                if extension == "svg" {
                    Some(entry)
                } else {
                    None
                }
            } else { None };
        })
        .map(|entry| entry.path().to_path_buf())
        .for_each(process_fn);
}

fn file_to_base64(
    file_path: &str
) -> io::Result<String> {
    let mut file = File::open(file_path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(general_purpose::STANDARD.encode(&content))
}


pub(crate) fn copy_static() {
    for entry in WalkDir::new("static").into_iter().filter_map(|e| e.ok()) {
        if let Some(extension) = entry.path().extension() {
            if extension == "svg" {
                let old_svg_path = entry.path()
                    .to_str()
                    .unwrap()
                    .to_string();
                let new_svg_path = entry.path()
                    .to_string_lossy()
                    .replace("static/", "build/original/svg/");

                let parent = Path::new(&new_svg_path)
                    .parent()
                    .expect("Error get parent dir");
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .expect("Unable to create directory");
                }

                fs::copy(old_svg_path.clone(), new_svg_path.clone()).expect("Couldn't copy file");

                println!("Copied: {} -> {}", old_svg_path, new_svg_path);
            }
        }
    }
}

fn generate_svg(
    organisation: &str,
    tera: &mut Tera,
    current: &&DescriptionObjects,
) {
    let mut filename = format!(
        "{}/{}/{}/{}.template.svg",
        "icons",
        organisation,
        current.zug,
        current.template
    );
    if !Path::new(&filename).exists() {
        filename = format!(
            "{}/{}/{}.template.svg",
            "icons",
            current.zug,
            current.template
        );
    }
    if !Path::new(&filename).exists() {
        println!("Skipping: {:?}", current.template);
    }

    current.names.split(",").for_each(|name| {
        vec![true, false].iter().for_each(|inverted| {
            current.special.split(",").for_each(|special| {
                let target_file_path = format!(
                    "{}{}.svg",
                    join_paths(vec!(
                        "build",
                        if *inverted { "inverted" } else { "original" },
                        "svg",
                        organisation,
                        &current.zug,
                        &uppercase_first_letter(&current.dir),
                    )),
                    join_filename(vec!(
                        name,
                        special,
                        &current.template
                    )),
                );

                process_file_common(
                    &filename,
                    &target_file_path,
                    organisation,
                    &*current.template,
                    *inverted,
                    name,
                    &*special,
                    "",
                    "",
                    &*current.dir,
                    tera.clone(),
                )
            });
        });
    });
}

fn process_file_common(
    file_path: &str,
    target_file_path: &str,
    organisation: &str,
    name: &str,
    inverted: bool,
    value: &str,
    special: &str,
    ort: &str,
    helfer: &str,
    dir: &str,
    tera: Tera,
) {
    println!("Processed content of {}", file_path);

    let mut context = Context::new();

    let main = match organisation.to_lowercase().as_str() {
        "thw" => { "#fff" }
        "fw" => { "#fff" }
        "zoll" => { "#fff" }
        "rettung" => { "#fff" }
        "pol" => { "#fff" }
        "bw" => { "#fff" }
        "alle" => { "#000" }
        &_ => { "#fff" }
    };
    let secondary = match organisation.to_lowercase().as_str() {
        "thw" => { "#003399" }
        "fw" => { "#FF0000" }
        "zoll" => { "#13A538" }
        "pol" => { "#13A538" }
        "bw" => { "#996633" }
        "rettung" => { "#000" }
        "kats" => { "#DF6711" }
        "alle" => { "#fff" }
        &_ => { "#000" }
    };


    context.insert("value", value);
    if organisation.to_lowercase() != "alle" {
        context.insert("organisation", &organisation.to_uppercase());
    } else {
        context.insert("organisation", "");
    }
    context.insert("ort", &ort);
    context.insert("helfer", &helfer);
    context.insert("special", &special);
    if inverted {
        context.insert("main_color", &secondary);
        context.insert("secondary_color", &main);
    } else {
        context.insert("main_color", &main);
        context.insert("secondary_color", &secondary);
    }

    let content = tera.render(
        format!(
            "{}/{}.template.svg",
            dir,
            name
        ).as_str(),
        &context,
    ).expect("Couldn't parse template");
    save_to_file(target_file_path, &content)
}

fn save_to_file(file_name: &str, content: &str) {
    println!("Saving  {}", file_name);


    let parent = Path::new(file_name).parent().expect("ERROR during path traversal");
    if !parent.exists() {
        fs::create_dir_all(parent).expect("Unable to create directory");
    }
    fs::write(file_name, content).expect("Unable to write file");
}

fn join_paths(
    paths: Vec<&str>,
) -> String {
    let mut string = paths
        .iter()
        .filter(|template| !template.is_empty())
        .map(|x| x.as_ref())
        .collect::<Vec<_>>()
        .join("/");
    string
        .push_str("/");
    string
}

fn join_filename(
    names: Vec<&str>,
) -> String {
    names
        .iter()
        .filter(|template| !template.is_empty())
        .map(|x| x.as_ref())
        .collect::<Vec<_>>()
        .join("-")
}

fn uppercase_first_letter(
    s: &str
) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

use std::{
    collections::HashMap,
    fs::{
        self
    },
    path::{Path, PathBuf},
    thread::{
        self,
        JoinHandle,
    },
    time::Duration,
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::iter::{
    IntoParallelIterator,
    ParallelIterator,
};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use walkdir::WalkDir;

use crate::config::{DescriptionObjects, Person};
use crate::utils::{
    calc_hash,
    create_progress_bar,
    file_to_base64,
    join_filename,
    join_paths,
    path_to_id,
    path_to_title,
    process_entries,
    save_to_file,
    uppercase_first_letter
};

mod svg_tools;
mod config;
mod utils;

fn main() {
    let (cfg, volunteer_config) = config::parse();

    let mut hashes: HashMap<String, String> = HashMap::new();
    if cfg.enable_png {
        let directory = Path::new("build");
        read_in_hashes(&mut hashes, directory);
    }

    let mut template_engine = match Tera::new("icons/**/*") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    template_engine.autoescape_on(vec![".template.svg"]);

    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim}[{pos:4} files][{elapsed:3}] {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let mut m = MultiProgress::new();


    let handler: JoinHandle<()>;
    if volunteer_config.enabled {
        let template_engine_clone = template_engine.clone();
        let pb;
        (pb, m) = create_progress_bar(&spinner_style, m, "volunteer", false);

        handler = thread::spawn(move || copy_volunteer(
            pb,
            template_engine_clone,
            &mut volunteer_config
                .personen
                .unwrap_or(vec! {}),
        ));
    } else {
        handler = thread::spawn(move || {})
    }
    let vec2: Vec<(Vec<DescriptionObjects>, &str)> = vec!(
        (cfg.thw, "THW"),
        (cfg.fw, "FW"),
        (cfg.pol, "POL"),
        (cfg.zoll, "Zoll"),
        (cfg.bw, "BW"),
        (cfg.rettung, "Rettung"),
        (cfg.kats, "KatS"),
        (cfg.alle, "Alle")
    );
    vec2
        .into_par_iter()
        .map(|(item, description)| {
            let pb = m.add(ProgressBar::new_spinner());
            pb.set_style(spinner_style.clone());
            pb.set_prefix(format!("[{:>7}]", description.to_string()));
            pb.enable_steady_tick(Duration::from_millis(100));

            generate_svg(
                pb.clone(),
                &item.clone(),
                description.to_string().clone(),
                &mut template_engine.clone(),
            );
        })
        .collect::<()>();

    let mut pb;
    (pb, m) = create_progress_bar(&spinner_style, m, "static", true);
    let handler2 = thread::spawn(move || copy_static(pb));

    let _ = handler.join();
    let _ = handler2.join();

    if cfg.enable_png {
        (pb, m) = create_progress_bar(&spinner_style, m, "png", false);

        svg_tools::convert_svg(pb, hashes)
    }
    let (pb, _m) = create_progress_bar(&spinner_style, m, "drawio", false);

    create_drawio(pb)
}


fn copy_volunteer(
    pb: ProgressBar,
    template_engine: Tera,
    volunteers: &mut Vec<Person>,
) {
    volunteers
        .iter()
        .map(|person| {
            person
                .volunteer
                .split(",")
                .map(|volunteer| (person, volunteer))
                .collect::<Vec<_>>()
        })
        .flatten()
        .map(|(person, volunteer)| {
            vec!(
                (person, volunteer, true),
                (person, volunteer, false)
            )
        })
        .flatten()
        .map(|(person, volunteer, inverted)| {
            person
                .value
                .split(",")
                .map(|special_position| (person, volunteer, inverted, special_position))
                .collect::<Vec<_>>()
        })
        .flatten()
        .for_each(|(person, volunteer, inverted, special_position)| {
            let target_file_path = format!(
                "build/custom/svg/{}/{}/{}/{}-{}-{}.svg",
                if inverted { "inverted" } else { "original" },
                &person.organisation,
                &person.zug,
                &volunteer,
                person.template,
                special_position,
            );

            pb.set_message(format!("Processed content of  {}", target_file_path));
            pb.inc(1);
            process_file_common(
                &target_file_path,
                &*person.organisation,
                &person.template,
                inverted,
                &*special_position,
                "",
                "",
                volunteer,
                "personen",
                template_engine.clone(),
            )
        });
    pb.finish_with_message("finished");
}

fn read_in_hashes(
    hashes: &mut HashMap<String, String>,
    directory: &Path,
) {
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_type = entry.file_type().unwrap();
                if file_type.is_dir() {
                    read_in_hashes(hashes, &entry.path());
                } else if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".svg") {
                        hashes.insert(
                            entry.path().to_str().unwrap().to_string(),
                            calc_hash(entry.path().to_str().unwrap())
                        );
                    }
                }
            }
        }
    } else {
        eprintln!("Error reading directory");
    }
}


#[derive(Serialize, Deserialize, Clone)]
struct DrawIoLibEntry {
    data: String,
    w: i32,
    h: i32,
    title: String,
    aspect: String,
}

fn create_drawio(
    pb: ProgressBar
) {
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

        pb.inc(1);
        pb.set_message(format!("Save to {}", format!("build/drawio/{}.xml", key).as_str()));
        save_to_file(
            format!("build/drawio/{}.xml", key).as_str(),
            format!("<mxlibrary>{}</mxlibrary>", &json_string).as_str(),
        )
    });
    pb.finish_with_message("finished")
}


pub(crate) fn copy_static(pb: ProgressBar) {
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

                pb.inc(1);
                pb.set_message(format!("Copied: {} -> {}", old_svg_path, new_svg_path));
            }
        }
    }
    pb.finish_with_message("finished")
}

fn generate_svg(
    pb: ProgressBar,
    vec: &Vec<DescriptionObjects>,
    organisation: String,
    tera: &mut Tera,
) {
    vec.iter().for_each(|current| {
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
            pb.set_message(format!("Skipping: {:?}", current.template));
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
                            organisation.as_str(),
                            &current.zug,
                            &uppercase_first_letter(&current.dir),
                        )),
                        join_filename(vec!(
                            name,
                            special,
                            &current.template
                        )),
                    );

                    pb.set_message(format!("Processed content of  {}", target_file_path));
                    pb.inc(1);
                    process_file_common(
                        &target_file_path,
                        organisation.as_str(),
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
    });
    pb.finish_with_message("finished");
}

fn process_file_common(
    target_file_path: &str,
    organisation: &str,
    name: &str,
    inverted: bool,
    value: &str,
    special: &str,
    ort: &str,
    volunteer: &str,
    dir: &str,
    tera: Tera,
) {
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
    context.insert("volunteer", &volunteer);
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

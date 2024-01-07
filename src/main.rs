use std::fs;
use std::path::Path;

use tera::{Context, Tera};
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


    let base_directory = "icons";


    cfg.thw.iter().for_each(|current| {
        generate_svg("THW", &mut tera, base_directory, &current);
    });
    cfg.fw.iter().for_each(|current| {
        generate_svg("FW", &mut tera, base_directory, &current);
    });
    cfg.pol.iter().for_each(|current| {
        generate_svg("POL", &mut tera, base_directory, &current);
    });
    cfg.zoll.iter().for_each(|current| {
        generate_svg("Zoll", &mut tera, base_directory, &current);
    });
    cfg.bw.iter().for_each(|current| {
        generate_svg("BW", &mut tera, base_directory, &current);
    });
    cfg.rettung.iter().for_each(|current| {
        generate_svg("Rettung", &mut tera, base_directory, &current);
    });
    cfg.kats.iter().for_each(|current| {
        generate_svg("KatS", &mut tera, base_directory, &current);
    });

    let thw_config = &helfer_config.personen;
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
                            base_directory,
                            &person.organisation,
                            val
                        );

                        process_file_helfer(
                            &filename,
                            &target_file_path,
                            &*person.organisation,
                            &person.template,
                            *inverted,
                            &*val,
                            helfer,
                            "personen",
                            tera.clone(),
                        );
                    });
                });
            });
        });
    }
    if cfg.enable_png {
        svg_tools::convert_svg()
    }
}

fn generate_svg(
    organisation: &str,
    tera: &mut Tera,
    base_directory: &str,
    current: &&DescriptionObjects,
) {
    let mut filename = format!(
        "{}/{}/{}/{}.template.svg",
        base_directory,
        organisation,
        current.zug,
        current.template
    );
    if !Path::new(&filename).exists() {
        filename = format!(
            "{}/{}/{}.template.svg",
            base_directory,
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

                process_file(
                    &filename,
                    &target_file_path,
                    organisation,
                    &*current.template,
                    *inverted,
                    name,
                    &*special,
                    &*current.dir,
                    tera.clone(),
                );
            });
        });
    });
}

fn process_file(
    file_path: &str,
    target_file_path: &str,
    organisation: &str,
    name: &str,
    inverted: bool,
    value: &str,
    special: &str,
    dir: &str,
    tera: Tera,
) {
    process_file_common(
        file_path,
        target_file_path,
        organisation,
        name,
        inverted,
        value,
        special,
        "",
        "",
        dir,
        tera,
    )
}

fn process_file_helfer(
    file_path: &str,
    target_file_path: &str,
    organisation: &str,
    name: &str,
    inverted: bool,
    value: &str,
    helfer: &str,
    dir: &str,
    tera: Tera,
) {
    process_file_common(
        file_path,
        target_file_path,
        organisation,
        name,
        inverted,
        value,
        "",
        "",
        helfer,
        dir,
        tera,
    )
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
        &_ => { "#000" }
    };


    context.insert("value", value);
    context.insert("organisation", organisation);
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
    ).expect("TODO: panic message");
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

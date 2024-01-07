use std::fs;
use std::path::Path;

use tera::{Context, Tera};

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
        let mut filename = format!(
            "{}/{}/{}/{}.template.svg",
            base_directory,
            "thw",
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
                            "thw",
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
                        "thw",
                        &*current.template,
                        *inverted,
                        name,
                        &*special,
                        tera.clone(),
                    );
                });
            });
        });
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

fn process_file(
    file_path: &str,
    target_file_path: &str,
    organisation: &str,
    name: &str,
    inverted: bool,
    value: &str,
    special: &str,
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
    tera: Tera,
) {
    println!("Processed content of {}", file_path);

    let mut context = Context::new();

    let thw_weiss = "#fff";
    let thw_blue = "#003399";


    context.insert("value", value);
    context.insert("ort", &ort);
    context.insert("helfer", &helfer);
    context.insert("special", &special);
    if inverted {
        context.insert("thw_main", &thw_blue);
        context.insert("thw_secondary", &thw_weiss);
    } else {
        context.insert("thw_main", &thw_weiss);
        context.insert("thw_secondary", &thw_blue);
    }

    let content = tera.render(format!("{}/{}.template.svg", organisation, name).as_str(), &context).expect("TODO: panic message");
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

fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

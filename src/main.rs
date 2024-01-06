use std::{fs};
use std::path::Path;
use tera::{Context, Tera};

mod svg_tools;
mod config;

fn main() {
    let (objects, config) = config::parse();


    let mut tera = match Tera::new("icons/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.autoescape_on(vec![".template.svg"]);


    let base_directory = "icons";


    objects.iter().for_each(|parsable_object| {
        let mut filename = format!(
            "{}/{}/{}/{}.template.svg",
            base_directory,
            &parsable_object.organisation,
            parsable_object.zug,
            parsable_object.type_object
        );
        if !Path::new(&filename).exists() {
            filename = format!(
                "{}/{}/{}.template.svg",
                base_directory,
                &parsable_object.organisation,
                parsable_object.type_object
            );
        }
        if !Path::new(&filename).exists() {
            println!("Skipping: {:?}", parsable_object);
        } else {
            parsable_object.value.iter().for_each(|val| {
                vec![true, false].iter().for_each(|inverted| {
                    let target_file_path = format!(
                        "build/{}/svg/{}/{}/{}-{}.svg",
                        if *inverted { "inverted" } else { "original" },
                        &parsable_object.organisation,
                        &parsable_object.zug,
                        parsable_object.type_object,
                        val
                    );


                    process_file(
                        &filename,
                        &target_file_path,
                        &*parsable_object.organisation,
                        &*parsable_object.type_object,
                        *inverted,
                        val,
                        tera.clone(),
                    );
                });
            });
        }
    });


    let thw_config = &config.personen;
    if config.enabled {
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
    //svg_tools::convert_svg()
}

fn process_file(
    file_path: &str,
    target_file_path: &str,
    organisation: &str,
    name: &str,
    inverted: bool,
    value: &str,
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

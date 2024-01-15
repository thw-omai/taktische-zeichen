use std::fs;
use std::path::Path;

use resvg::{tiny_skia, Tree, usvg};
use resvg::usvg::{fontdb, TextRendering, TreeParsing, TreeTextToPath};
use tiny_skia::{Pixmap, Transform};
use usvg::Options;

use crate::process_entries;

pub(crate) fn convert_svg() {
    process_entries("build", |entry| {
        let svg_path = entry.to_str().unwrap().to_string();
        let png_path = format!(
            "{}.png",
            entry.with_extension("").to_str().unwrap()
        ).replace("svg/", "png/");

        let parent = Path::new(&png_path).parent().expect("Error get parent dir");
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Unable to create directory");
        }

        convert_svg_to_png(&svg_path, &png_path);

        println!("Converted: {} -> {}", svg_path, png_path);
    });
}

fn convert_svg_to_png(
    svg_path: &str,
    png_path: &str,
) {
    let mut opt = Options::default();
    opt.text_rendering = TextRendering::GeometricPrecision;

    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();
    fontdb.load_fonts_dir("./fonts/ttf");

    let svg_data = fs::read(svg_path).unwrap();
    let mut tree_usvg = usvg::Tree::from_data(&svg_data, &opt).unwrap();
    tree_usvg.convert_text(&fontdb);


    let tree = Tree::from_usvg(&tree_usvg);

    let pixmap_size = tree.size.to_int_size();
    let mut pixmap = Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

    Tree::render(&tree, Transform::default(), &mut pixmap.as_mut());

    pixmap.save_png(png_path).unwrap();
}

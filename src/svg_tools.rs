use std::{
    fs,
    thread,
    path::{Path, PathBuf},
};
use std::collections::HashMap;
use indicatif::ProgressBar;

use resvg::{
    tiny_skia,
    Tree,
    usvg::{
        self,
        fontdb,
        Size,
        TextRendering,
        TreeParsing,
        TreeTextToPath,
    },
};
use tiny_skia::{Pixmap, Transform};
use usvg::Options;

use crate::process_entries;
use crate::utils::calc_hash;

pub(crate) fn convert_svg(
    progress_bar: ProgressBar,
    hashes: HashMap<String, String>,
) {
    let mut v = Vec::<thread::JoinHandle<()>>::new();
    process_entries("build", |entry: PathBuf| {
        let pb = progress_bar.clone();
        let cloned_hashes = hashes.clone();
        let jh = thread::spawn(move || {
            let binding = cloned_hashes.clone();
            let old_checksum = match binding.get(entry.to_str().unwrap()) {
                Some(x) => x,
                None => "CREATE",
            };
            let new_checksum = calc_hash(entry.to_str().unwrap());

            if old_checksum != new_checksum {
                let svg_path = entry
                    .to_str()
                    .unwrap()
                    .to_string();
                let original_png_path = format!(
                    "{}.png",
                    entry
                        .with_extension("")
                        .to_str()
                        .unwrap()
                );

                [128, 256, 512, 1024, 2048].iter().for_each(|size| {
                    let png_path = original_png_path.replace(
                        "svg/",
                        &*format!("png/{}/", size.to_string().as_str())
                    );

                    convert_svg_to_png(
                        &svg_path,
                        &png_path,
                        *size as f32
                    );

                    pb.inc(1);
                    pb.set_message(format!("Converted: {} -> {}", svg_path, png_path));
                });
            }
        });
        v.push(jh);
    });

    for jh in v.into_iter() {
        jh.join().unwrap();
    }
    progress_bar.finish_with_message("finished");
}

fn convert_svg_to_png(
    svg_path: &str,
    png_path: &str,
    size: f32,
) {
    fs::create_dir_all(Path::new(png_path).parent().unwrap()).expect("Couldn't create directory");
    let mut opt = Options::default();
    opt.text_rendering = TextRendering::GeometricPrecision;

    opt.default_size = Size::from_wh(size, size).unwrap();

    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();
    fontdb.load_fonts_dir("./fonts/ttf");

    let svg_data = fs::read(svg_path).unwrap();
    let mut tree_usvg = usvg::Tree::from_data(&svg_data, &opt).unwrap();
    tree_usvg.convert_text(&fontdb);
    tree_usvg.size = Size::from_wh(size, size).unwrap();

    let tree = Tree::from_usvg(&tree_usvg);

    let pixmap_size = tree.size.to_int_size();
    let mut pixmap = Pixmap::new(
        pixmap_size.width(),
        pixmap_size.height(),
    ).unwrap();

    Tree::render(
        &tree,
        Transform::default(),
        &mut pixmap.as_mut(),
    );

    pixmap.save_png(png_path).unwrap();
}

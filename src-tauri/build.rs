use std::{env, fs, path::Path};

fn main() {
    bundle_locales();
    tauri_build::build()
}

fn bundle_locales() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let build_path = Path::new(&out_dir).parent().unwrap().parent().unwrap();

    let bundle_path = build_path
        .read_dir()
        .expect("Failed to read directory")
        .filter_map(|e| e.ok())
        .find_map(|e| {
            let p = e.path();
            p.file_name()?
                .to_str()?
                .starts_with("tauri-plugin-i18n")
                .then_some(p)
        })
        .map(|p| p.join("out").join("bundled_locales.rs"));

    let Some(bundle_path) = bundle_path else {
        eprintln!("No tauri-plugin-i18n found, skipping");
        return;
    };

    let manifest_dir = env::var("CARGO_MANIFEST_PATH").unwrap();
    let locales_path = Path::new(&manifest_dir).parent().unwrap().join("locales");

    if !locales_path.exists() {
        panic!(
            "Locales directory does not exist: {}",
            locales_path.display()
        );
    }

    println!("cargo:rerun-if-changed={}", locales_path.display());

    let mut code = String::from("pub fn get_bundled_data() -> Vec<(&'static str, &'static str, &'static str)> {\n    vec![\n");

    let mut entries: Vec<_> = fs::read_dir(&locales_path)
        .expect("Failed to read locales")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let count = entries.len();

    for entry in entries {
        let path = entry.path();
        let (Some(stem), Some(ext)) = (
            path.file_stem().and_then(|s| s.to_str()),
            path.extension().and_then(|s| s.to_str()),
        ) else {
            continue;
        };

        println!("cargo:info=  Bundling: {}.{}", stem, ext);
        code.push_str(&format!(
            "        ({:?}, {:?}, include_str!(r#\"{}\"#)),\n",
            stem,
            ext,
            path.display()
        ));
    }

    println!("cargo:info=Successfully bundled {} locale file(s)", count);
    code.push_str("    ]\n}\n");

    if let Some(parent) = bundle_path.parent() {
        if parent.exists() {
            fs::write(&bundle_path, code).expect("Failed to write bundled_locales.rs");
        }
    }
}

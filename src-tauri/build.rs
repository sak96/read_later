use std::{env, fs, path::Path};

fn main() {
    bundle_locales();
    tauri_build::build()
}

fn bundle_locales() {
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:info=OUT_DIR: {}", out_dir);
    let build_path = Path::new(&out_dir).parent().unwrap().parent().unwrap();
    for bundle_path in fs::read_dir(build_path)
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("tauri-plugin-i18n"))
                .unwrap_or(false)
        })
        .map(|p| p.join("out").join("bundled_locales.rs"))
    {
        println!("cargo:info=bundle_path: {:?}", bundle_path);

        let manifest_dir = env::var("CARGO_MANIFEST_PATH").unwrap();
        println!("cargo:info=manifest {}", manifest_dir);
        let locales_path = Path::new(&manifest_dir).parent().unwrap().join("locales");
        println!("cargo:info=locale path {:?}", locales_path);

        if !locales_path.exists() {
            panic!(
                "Locales directory does not exist: {}",
                locales_path.display()
            );
        }

        println!("cargo:rerun-if-changed={}", locales_path.display());

        let mut code = String::from(
            "pub fn get_bundled_data() -> Vec<(&'static str, &'static str, &'static str)> {\n    vec![\n",
        );
        match fs::read_dir(&locales_path) {
            Ok(entries) => {
                let mut count = 0;
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && let (Some(stem), Some(ext)) = (
                            path.file_stem().and_then(|s| s.to_str()),
                            path.extension().and_then(|s| s.to_str()),
                        )
                    {
                        count += 1;
                        println!("cargo:info=  Bundling: {}.{}", stem, ext);
                        code.push_str(&format!(
                            "        ({:?}, {:?}, include_str!(r#\"{}\"#)),\n",
                            stem,
                            ext,
                            path.display()
                        ));
                    }
                }
                println!("cargo:info=Successfully bundled {} locale file(s)", count);
            }
            Err(e) => {
                panic!(
                    "Failed to read locales directory at {}: {}",
                    locales_path.display(),
                    e
                );
            }
        }
        code.push_str("    ]\n}\n");
        if let Some(parent) = bundle_path.parent() {
            if parent.exists() {
                fs::write(&bundle_path, code).expect("Failed to write bundled_locales.rs");
            } else {
                eprintln!("Skipping write, parent does not exist: {:?}", parent);
            }
        } else {
            eprintln!("Skipping write, no parent for path: {:?}", bundle_path);
        }
    }
}

use std::{
    fs::File,
    io::{BufWriter, Read, Write},
};

fn build_slang(file: &str) {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let output_spv = out_dir.join(format!("{}.spv", file));
    let output_json = out_dir.join(format!("{}.json", file));
    let output_rs = out_dir.join(format!("{}_reflection.rs", file));
    let output = std::process::Command::new("slangc")
        .args([
            &format!("{}/../../shaders/{}.slang", manifest_dir, file),
            "-o",
        ])
        .arg(&output_spv)
        .arg("-reflection-json")
        .arg(&output_json)
        .output()
        .expect("Failed to run slangc");

    if !(output.status.success()) {
        eprintln!("slangc failed:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        if !output.stdout.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        }
        panic!("Shader compilation failed.");
    }

    // Turn reflected json into a .rs file with a const time hashmap from phf_codegen
    let mut json = File::open(output_json).unwrap();
    let mut text = String::new();
    json.read_to_string(&mut text).unwrap();
    let out = slang::generate_hashmap(&text).unwrap();
    let mut out_file = BufWriter::new(File::create(&output_rs).unwrap());
    let mut cg = phf_codegen::Map::new();
    for (k, v) in out {
        cg.entry(format!("{}.bind_group", k), v.bind_group.to_string());
        cg.entry(format!("{}.bind_index", k), v.bind_index.to_string());
        cg.entry(format!("{}.byte_offset", k), v.byte_offset.to_string());
    }
    write!(
        &mut out_file,
        "pub static {}_REFLECTION: phf::Map<&'static str, usize> = {};\n",
        file.to_ascii_uppercase(),
        cg.build()
    )
    .unwrap();
}

fn main() {
    println!("cargo:rerun-if-changed=shaders");

    build_slang("render");

    // Old WESL stuff:
    wesl::Wesl::new("src/shaders")
        .build_artifact(&"package::extension".parse().unwrap(), "extension");
    wesl::Wesl::new("src/shaders").build_artifact(&"package::new_ray".parse().unwrap(), "new_ray");
    wesl::Wesl::new("src/shaders").build_artifact(&"package::logic".parse().unwrap(), "logic");
    wesl::Wesl::new("src/shaders").build_artifact(&"package::shader".parse().unwrap(), "shader");
}

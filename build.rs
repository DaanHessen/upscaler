use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("model.onnx");

    if !dest_path.exists() {
        println!("cargo:warning=Downloading NSFW model (model.onnx) to OUT_DIR...");
        
        let url = "https://github.com/Fyko/nsfw/releases/download/v0.2.0/model.onnx";
        
        match ureq::get(url).call() {
            Ok(response) => {
                let mut dest = fs::File::create(&dest_path)
                    .unwrap_or_else(|e| panic!("Failed to create model.onnx: {}", e));
                
                let mut reader = response.into_body().into_reader();
                std::io::copy(&mut reader, &mut dest)
                    .unwrap_or_else(|e| panic!("Failed to download model.onnx: {}", e));
            }
            Err(e) => panic!("Error downloading model: {}", e),
        }
    } else {
        println!("cargo:warning=NSFW model already exists in OUT_DIR, ready to load.");
    }
}

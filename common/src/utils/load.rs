use std::path::PathBuf;

pub fn load_resource_path(relative_path: &str) -> &'static str {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = PathBuf::from(manifest_dir).join(relative_path);
    Box::leak(
        std::fs::read_to_string(full_path)
            .expect("Failed to read resource")
            .into_boxed_str(),
    )
}

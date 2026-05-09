// Ensure ../../frontend/dist/ exists so rust-embed's compile-time scan
// doesn't fail on a fresh checkout. The directory is gitignored (the actual
// build artefacts inside live there only after `pnpm build`); creating it
// here gives rust-embed an empty folder to embed when the frontend hasn't
// been built yet, and the viewer route falls back to a friendly 404 hint.

fn main() {
    let dist = std::path::Path::new("../../frontend/dist");
    if !dist.exists() {
        let _ = std::fs::create_dir_all(dist);
    }
    println!("cargo:rerun-if-changed=../../frontend/dist");
    println!("cargo:rerun-if-changed=build.rs");
}

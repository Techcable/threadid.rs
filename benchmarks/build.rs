pub fn main() {
    println!("cargo:rustc-check-cfg=cfg(has_thread_current_id)");
    println!("cargo:rustc-check-cfg=cfg(nightly)");
    if rustversion::cfg!(nightly) {
        println!("cargo:rustc-cfg=has_thread_current_id");
        println!("cargo:rustc-cfg=nightly");
    }
}

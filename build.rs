use cc;

fn main() {
    cc::Build::new().file("cc/helper.c").compile("helper");

    // TODO: fill in more architectures where chars are unsigned
    #[cfg(target_arch="arm")]
    println!("cargo:rustc-cfg=char_is_unsigned");

    println!("cargo:rerun-if-changed=cc/helper.c");
}

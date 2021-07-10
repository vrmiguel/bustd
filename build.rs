use cc;

fn main() {
    cc::Build::new().file("cc/helper.c").compile("helper");

    println!("cargo:rerun-if-changed=cc/helper.c");
}

fn main() {
    let theos = env!("THEOS");
    println!("cargo:rustc-link-search=framework={}/vendor/lib/", theos);
}


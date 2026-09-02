fn main() {
    // `option_env!` alone won't rebuild when the value changes; this makes cargo
    // re-evaluate API_BASE whenever PLAP_API_BASE is set/changed/unset.
    println!("cargo:rerun-if-env-changed=PLAP_API_BASE");
}

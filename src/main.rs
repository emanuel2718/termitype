fn main() {
    if let Err(err) = termitype::run() {
        eprintln!("termitype: {err:?}");
        std::process::exit(1)
    }
}

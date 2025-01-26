fn main() {
    if let Err(err) = termitype::run() {
        eprintln!("Termitype error: {:?}", err);
        std::process::exit(1)
    }
}

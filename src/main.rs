fn main() {
    if let Err(err) = termitype::start() {
        eprint!("Something went wrong starting termitype: {err:?}");
        std::process::exit(1)
    }
}

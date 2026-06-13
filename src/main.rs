fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    // Handle -v/--version and -V flags
    if args.len() == 2 && (args[1] == "-v" || args[1] == "-V" || args[1] == "--version") {
        println!("soupify {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }
    
    if let Err(error) = soupify::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

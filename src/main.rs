// This is a very basic CLI, mainly intended to be used in non-rust applications.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (release, language, arch) = match args.len() {
        2 => (args[1].as_str(), "English (United States)", "x86_64"),
        3 => (args[1].as_str(), args[2].as_str(), "x86_64"),
        4 => (args[1].as_str(), args[2].as_str(), args[3].as_str()),
        _ => {
            eprintln!("Usage: {} [language] [arch]", args[0]);
            std::process::exit(1);
        }
    };

    match rido::WindowsRelease::new(release, language, arch) {
        Ok(release) => println!("{} {}", &release.url, if let Some(hash) = &release.hash { hash } else { "" }),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
    

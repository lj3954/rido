const LICENSE: &str = include_str!("../LICENSE");

fn main() -> Result<(), rido::RidoError> {
    let args: Vec<String> = std::env::args().collect();
    let (release, language, arch) = match args.len() {
        2 => (args[1].as_str(), "English (United States)", "x86_64"),
        3 => (args[1].as_str(), args[2].as_str(), "x86_64"),
        4 => (args[1].as_str(), args[2].as_str(), args[3].as_str()),
        _ => {
            println!(
                "This program is free software: you can redistribute it and/or modify it under the 
terms of the GNU General Public License as published by the Free Software Foundation,  either 
version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See 
the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program.  If 
not, see <https://www.gnu.org/licenses/>.

The source code for this program can be found at 'https://github.com/lj3954/rido'.
For more information, run {} --license\n",
                args[0]
            );
            eprintln!("Usage: {} [release] [language] [arch]", args[0]);
            std::process::exit(1);
        }
    };

    if release == "--license" {
        println!("{}", LICENSE);
        std::process::exit(0);
    }

    match rido::WindowsData::new(release, language, arch) {
        Ok(release) => {
            if let Some(hash) = &release.hash {
                println!("{} {}", release.url, hash);
            } else {
                println!("{}", release.url);
            }
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    Ok(())
}

/* artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */
//! The CLI main binary


#[macro_use]
extern crate ergo;

#[macro_use]
extern crate quicli;

use ergo::*;
use quicli::prelude::*;


#[derive(Debug, StructOpt)]
#[structopt(name="ls", about="List and filter artifacts")]
pub(crate) struct Ls {
    #[structopt(help="Pattern to search for")]
    pattern: String,
}

pub fn run() -> Result<()> {
    use quicli::prelude::structopt::clap::*;
    let app = structopt::clap::App::new("art")
        .author("github.com/vitiral/artifact")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Design documentation tool for everybody.")
        .arg(Arg::with_name("verbose")
             .short("v")
             .multiple(true)
             .help("Verbose, pass up to 4 times to increase the level")
        )
        .subcommand(Ls::clap());

    let matches = app.get_matches();
    set_log_verbosity(matches.occurrences_of("verbose"))?;

    match matches.subcommand() {
        ("ls", Some(args)) => {
            let ls = Ls::from_clap(args.clone());
            eprintln!("GOT ls:\n{:#?}", ls);
            eprintln!("pattern: {}", ls.pattern);
        },
        (sub, _) => unimplemented!("sub: {}", sub),
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            ::std::process::exit(1);
        }
    }
}

use std::io;
use std::io::Write;
use std::fmt::Write as FmtWrite;
use std::iter::FromIterator;
use std::process::exit;
use std::collections::HashSet;

use regex::{Regex, RegexBuilder};
use clap::{Arg, App, SubCommand, ArgMatches, AppSettings as AS};

use core::fmt::{FmtSettings, fmt_artifact};
use core::{Artifacts, ArtName, parse_names, Settings};
use cmdline::search::{VALID_SEARCH_FIELDS, SearchSettings, PercentSearch, show_artifact};


pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    // TODO: implement -c and -t
    SubCommand::with_name("ls")
        .about("list artifacts according to various parameters")
        .settings(&[AS::DeriveDisplayOrder, AS::ColoredHelp])
        .arg(Arg::with_name("search")
                 .help("artifact names given in form REQ-foo-[bar, baz-[1,2]] OR pearl regexp \
                        pattern if -p is given")
                 .use_delimiter(false))
        .arg(Arg::with_name("pattern")
                 .short("p")
                 .help("SEARCH using a pearl regexp pattern in the given FIELDS instead of \
                        searching by name. Valid areas are: N=name, D=path, P=parts, O=partof, \
                        L=loc, R=refs, T=text, A=see '-A'")
                 .value_name("FIELDS")
                 .takes_value(true)
                 .max_values(1))
        .arg(Arg::with_name("long")
                 .short("l")
                 .help("print items in the 'long form'"))
        .arg(Arg::with_name("recursive")
                 .short("r")
                 .help("print the parts of the artifact up to the given depth (default 1)")
                 .value_name("DEPTH")
                 .takes_value(true)
                 .validator(|s| {
                     match s.parse::<u8>() {
                         Ok(_) => Ok(()),
                         Err(e) => Err(e.to_string()),
                     }
                 })
                 .default_value("0")
                 .max_values(1))
        .arg(Arg::with_name("completed")
                 .short("c")
                 .help("give a filter for the completedness in %. I.e. '<45'. '<' is the default \
                        if no comparison operator is given, '<0' is the default if no args are \
                        given. Note: all comparisons are inclusive (<1 means <=1)")
                 .takes_value(true))
        .arg(Arg::with_name("tested")
                 .short("t")
                 .help("give a filter for the testedness in %. see '-c'")
                 .takes_value(true))
        .arg(Arg::with_name("all")
                 .short("A")
                 .help("activate all display flags. If this flag is set, additional flags will \
                        be *deactivated* instead of activated"))
        .arg(Arg::with_name("path")
                 .short("D")
                 .help("display the path where the artifact is defined"))
        .arg(Arg::with_name("parts")
                 .short("P")
                 .help("display the parts of the artifact"))
        .arg(Arg::with_name("partof")
                 .short("O")
                 .help("display the artifacts which this artifact is a partof"))
        .arg(Arg::with_name("loc")
                 .short("L")
                 .help("display location name"))
        .arg(Arg::with_name("refs")
                 .short("R")
                 .help("display the references to this artifact"))
        .arg(Arg::with_name("text")
                 .short("T")
                 .help("display the text description of this artifact (first line only if not -l)"))
}

/// return (lt, percent) returning None when there is no value
pub fn _get_percent(s: &str) -> Result<(Option<bool>, Option<u8>), String> {
    let mut s = s;
    let mut lt = None;
    if s.len() == 0 {
        return Ok((lt, None));
    }
    let mut had_sign = true;
    match s.chars().next().unwrap() {
        '<' => lt = Some(true),
        '>' => lt = Some(false),
        '0'...'9' => had_sign = false,
        _ => {
            return Err("percent must be of the form: [SIGN]NUM where NUM is between 0 and 100 and \
                        SIGN is an optional < or >"
                           .to_string())
        }
    }
    if had_sign {
        // the first char was either < or >
        s = s.split_at(1).1;
        if s.len() == 0 {
            return Ok((lt, None));
        }
    }
    if s.len() == 0 {
        return Ok((lt, None));
    }
    match s.parse::<u8>() {
        Ok(v) => {
            if v <= 100 {
                Ok((lt, Some(v)))
            } else {
                Err("NUM must be between 0 and 100".to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

fn get_percent(s: &str) -> Result<PercentSearch, String> {
    Ok(match _get_percent(s) {
        Ok((lt, perc)) => {
            if lt.is_none() && perc.is_none() {
                PercentSearch {
                    lt: false,
                    perc: 100,
                }
            } else if perc.is_none() {
                match lt.unwrap() {
                    true => {
                        PercentSearch {
                            lt: true,
                            perc: 0,
                        }
                    }
                    false => {
                        PercentSearch {
                            lt: false,
                            perc: 100,
                        }
                    }
                }
            } else {
                let lt = match lt {
                    None => false,
                    Some(l) => l,
                };
                let perc = match perc {
                    None => 100,
                    Some(p) => p,
                };
                PercentSearch {
                    lt: lt,
                    perc: perc,
                }
            }
        }
        Err(e) => return Err(e),
    })
}

#[test]
fn test_get_percent() {
    // correct
    assert_eq!(_get_percent(""), Ok((None, None)));
    assert_eq!(_get_percent("<"), Ok((Some(true), None)));
    assert_eq!(_get_percent(">"), Ok((Some(false), None)));
    assert_eq!(_get_percent("<10"), Ok((Some(true), Some(10))));
    assert_eq!(_get_percent(">100"), Ok((Some(false), Some(100))));

    // test full struct
    assert_eq!(get_percent(""),
               Ok(PercentSearch {
                   lt: false,
                   perc: 100,
               }));
    assert_eq!(get_percent("<"),
               Ok(PercentSearch {
                   lt: true,
                   perc: 0,
               }));
    assert_eq!(get_percent(">"),
               Ok(PercentSearch {
                   lt: false,
                   perc: 100,
               }));
    assert_eq!(get_percent("89"),
               Ok(PercentSearch {
                   lt: false,
                   perc: 89,
               }));
    assert_eq!(get_percent(">89"),
               Ok(PercentSearch {
                   lt: false,
                   perc: 89,
               }));
    assert_eq!(get_percent("<89"),
               Ok(PercentSearch {
                   lt: true,
                   perc: 89,
               }));

    // invalid
    assert!(get_percent(">101").is_err());
    assert!(get_percent(">-1").is_err());
    assert!(get_percent("a").is_err());
    assert!(get_percent("<a").is_err());

}

/// get all the information from the user input
pub fn get_ls_cmd(matches: &ArgMatches) -> Result<(String, FmtSettings, SearchSettings), String> {
    let mut settings = FmtSettings::default();
    settings.long = matches.is_present("long");
    settings.recurse = matches.value_of("recursive").unwrap().parse::<u8>().unwrap();
    settings.path = matches.is_present("path");
    settings.parts = matches.is_present("parts");
    settings.partof = matches.is_present("partof");
    settings.loc_path = matches.is_present("loc");
    settings.refs = matches.is_present("refs");
    settings.text = matches.is_present("text");
    if matches.is_present("all") {
        // reverse everything
        settings.path = !settings.path;
        settings.parts = !settings.parts;
        settings.partof = !settings.partof;
        settings.loc_path = !settings.loc_path;
        settings.refs = !settings.refs;
        settings.text = !settings.text;
    } else if settings.long &&
       !(settings.path || settings.parts || settings.partof || settings.loc_path ||
         settings.refs || settings.text) {
        // if long is specified but no other display attributes are specified
        settings.path = true;
        settings.parts = true;
        settings.partof = true;
        settings.refs = true;
        settings.text = true;
    }
    let mut search_settings;
    match matches.value_of("pattern") {
        Some(p) => {
            search_settings = try!(SearchSettings::from_regex(p));
            search_settings.use_regex = true;
        }
        None => search_settings = SearchSettings::new(),
    }
    debug!("tested: {:?}", search_settings.tested);
    match matches.value_of("completed") {
        Some(c) => search_settings.completed = try!(get_percent(c)),
        None => {}
    }
    match matches.value_of("tested") {
        Some(t) => {
            debug!("got tested: {}", t);
            search_settings.tested = try!(get_percent(t));
        }
        None => {}
    }
    debug!("tested: {:?}", search_settings.tested);

    let search = matches.value_of("search").unwrap_or("").to_string();

    debug!("ls search: {}, settings: {:?}, search_settings: {:?}",
           search,
           settings,
           search_settings);
    Ok((search, settings, search_settings))
}

/// perform the ls command given the inputs
pub fn do_ls(search: String,
             artifacts: &Artifacts,
             fmtset: &FmtSettings,
             search_set: &SearchSettings,
             settings: &Settings) {
    let mut dne: Vec<ArtName> = Vec::new();
    let mut names = Vec::new();
    let mut pat: Option<Regex> = None;
    let pat_case;
    if search_set.use_regex {
        // names to use are determined by filtering the regex
        let pat = RegexBuilder::new(&search)
                      .case_insensitive(true)
                      .compile();
        pat_case = match pat {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid pattern: {}", e.to_string());
                exit(1);
            }
        };
    } else {
        // names to use are determined from the beginning
        names.extend(parse_names(&search).unwrap());
        names.sort();
        debug!("artifact names selected: {:?}", names);
        pat_case = Regex::new("").unwrap();
    }
    if names.len() == 0 {
        names.extend(artifacts.keys().map(|n| n.clone()));
        names.sort();
    }

    let mut displayed: HashSet<ArtName> = HashSet::new();
    let mut stdout = io::stdout();
    for name in names {
        let art = match artifacts.get(&name) {
            Some(a) => a,
            None => {
                dne.push(name);
                continue;
            }
        };
        if !show_artifact(&name, art, &pat_case, search_set) {
            continue;
        }
        let f = fmt_artifact(&name, artifacts, fmtset, fmtset.recurse, &mut displayed);
        f.write(&mut stdout, artifacts, settings, 0).unwrap(); // FIXME: unwrap
    }
    if dne.len() > 0 {
        error!("The following artifacts do not exist: {:?}", dne);
        exit(1);
    }
}
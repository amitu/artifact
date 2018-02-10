use artifact_data;

#[allow(unused_imports)]
use ergo::*;
#[allow(unused_imports)]
use quicli::prelude::*;


#[derive(Debug, StructOpt)]
#[structopt(name = "ls", about = "List and filter artifacts")]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub struct Ls {
    /// Pass many times for more log output.
    #[structopt(long = "verbose", short = "v")]
    pub verbosity: u64,

    #[structopt(name="PATTERN", help = "\
Regular expression to search for artifact names.")]
    pub pattern: String,

    #[structopt(short="f", long="fields", value_name="FIELDS",
      default_value="name,parts",
      help="\
Specify fields to search for the regular expression PATTERN.

Valid FIELDS are:
- N/name: search the \"name\" field (artifact name)
- F/file: search the \"file\" field (see -F)
- P/parts: search the \"parts\" field (see -P)
- O/partof: search the \"partof\" field (see -O)
- C/code: search the \"code\" field (see -C)
- T/text: search the \"text\" field (see -T)

Fields can be listed by all caps, or comma-separated lowercase.

Both of these commands will list only artifacts with \"foobar\" in the name
or text fields of all artifacts.

    art ls foobar -p NT
    art ls foobar -p name,text

Regular expressions use the rust regular expression syntax, which is almost
identical to perl/python with a few minor differences

https://doc.rust-lang.org/regex/regex/index.html#syntax.\n\n    ")]
    pub fields: String,

    #[structopt(short="l", long="long", help = "Print items in the 'long form'")]
    pub long: bool,


    #[structopt(short="s", long="spc", default_value=">0", help = "\
Filter by spc (specification) completeness
- `-s \"<45\"`: show only items with spc <= 45%.
- `-s \">45\"`: show only items with spc >= 45%.
- `-s \"<\"`  : show only items with spc <=  0%.
- `-s \">\"`  : show only items with spc >=100%\n\n    ")]
    pub spc: String,

    #[structopt(short="t", long="tst", default_value=">0", help = "\
Filter by tst (test) completeness. See `-s/--spc` for format.")]
    pub tst: String,

    #[structopt(short="F", long="file", help = "\
\"file\" field: show the file where the artifact is defined.")]
    pub file: bool,

    #[structopt(short="P", long="parts", help = "\
\"parts\" field: show the children of the artifact.")]
    pub parts: bool,

    #[structopt(short="O", long="partof", help = "\
\"partof\" field: show the parents of the artifact.")]
    pub partof: bool,

    #[structopt(short="C", long="code", help = "\
\"code\" field: show the code paths where the artifact is implemented.")]
    pub code: bool,

    #[structopt(short="T", long="text", help = "\
\"text\" field: show the text of the artifact")]
    pub text: bool,

    #[structopt(short="A", long="all", help = "\
\"all\" field: activate ALL fields, additional fields DEACTIVATE fields")]
    pub all: bool,

    #[structopt(long="plain", help = "Do not display color in the output.")]
    pub plain: bool,

    #[structopt(long="type", default_value="list", help = "\
Type of output from [list, json]")]
    pub ty_: String,

    #[structopt(long="work-dir", help = "Use a different working directory [default: $CWD]")]
    pub work_dir: Option<String>,
}

/// Run the `art ls` command
pub fn run(cmd: Ls) -> Result<i32> {
    set_log_verbosity("art", cmd.verbosity)?;
    let work_dir = match cmd.work_dir {
        Some(d) => PathDir::new(d),
        None => PathDir::current_dir(),
    }?;
    info!("Running art-ls in working directory {}", work_dir.display());
    Ok(0)
}

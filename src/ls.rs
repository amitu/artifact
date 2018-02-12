use dev_prelude::*;
use artifact_data::*;
use termstyle::{self, El, Text, Color};

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

    #[structopt(short="N", long="name", help = "\
\"name\" field: show the name of the artifact.")]
    pub name: bool,

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
    let work_dir = work_dir!(cmd);
    info!("Running art-ls in working directory {}", work_dir.display());

    let (mut lints, project) = read_project(work_dir)?;
    Ok(0)
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Flags {
    name: bool,
    file: bool,
    parts: bool,
    partof: bool,
    code: bool,
    text: bool,
}

lazy_static!{
    pub static ref VALID_SEARCH_FIELDS: OrderSet<&'static str> = OrderSet::from_iter(
        ["N", "F", "P", "O", "C", "T", "A",
        "name", "file", "parts", "partof", "code", "text", "all"]
        .iter().map(|s| *s));

    pub static ref ANY_UPPERCASE: Regex = Regex::new("[A-Z]").unwrap();
}

impl Default for Flags {
    fn default() -> Flags {
        Flags {
            name: true,
            file: false,
            parts: true,
            partof: false,
            code: false,
            text: false,
        }
    }
}

impl Flags {
    pub fn from_str<'a>(s: &'a str) -> Result<Flags> {
        if s.is_empty() {
            return Ok(Flags::default());
        }
        let first_char = s.chars().next().unwrap();
        let flags: OrderSet<&'a str> = if s.contains(',') {
            s.split(',').filter(|s| !s.is_empty()).collect()
        } else if ANY_UPPERCASE.find(s).is_none() {
            orderset!(s)
        } else {
            s.split("").filter(|s| !s.is_empty()).collect()
        };

        let invalid: OrderSet<&'a str> =
            flags.difference(&VALID_SEARCH_FIELDS).map(|s| *s).collect();
        ensure!(invalid.is_empty(), "Unknown fields: {:#?}", invalid);
        let fc = |c| flags.contains(c);
        let all = fc("A") || fc("all");
        let out = Flags {
            name: fc("N") || fc("name"),
            file: fc("F") || fc("file"),
            parts: fc("P") || fc("parts"),
            partof: fc("O") || fc("partof"),
            code: fc("C") || fc("code"),
            text: fc("T") || fc("text"),
        };
        Ok(out.resolve_actual(all))
    }

    /// Get the given flags from the command
    pub fn from_cmd(cmd: Ls) -> Flags {
        let out = Flags {
            name: cmd.name,
            file: cmd.file,
            parts: cmd.parts,
            partof: cmd.partof,
            code: cmd.code,
            text: cmd.text,
        };
        out.resolve_actual(cmd.all)
    }

    /// Flags with `all` taken into account.
    ///
    /// If no flags are set then use the default.
    fn resolve_actual(self, all: bool) -> Flags {
        if all {
            self.invert()
        } else if self.is_empty() {
            Flags::default()
        } else {
            self
        }
    }

    /// Return whether no flags are set
    pub fn is_empty(&self) -> bool {
        !(self.name || self.file || self.parts || self.partof || self.code || self.text)
    }

    /// Invert the flag selection.
    fn invert(&self) -> Flags {
        Flags {
            name: !self.name,
            file: !self.file,
            parts: !self.parts,
            partof: !self.partof,
            code: !self.code,
            text: !self.text,
        }
    }
}

trait ArtifactExt {
    fn line_style(&self, flags: &Flags, plain: bool) -> Vec<Vec<Text>>;
    fn name_style(&self) -> Text;
}

impl ArtifactExt for Artifact {
    fn line_style(&self, flags: &Flags, plain: bool) -> Vec<Vec<Text>> {
        // first col: spc+tst
        let mut out = vec![
            vec![
                self.completed.spc_style(),
                Text::new(" ".into()),
                self.completed.tst_style(),
            ],
        ];

        if flags.name {
            out.push(vec![self.name_style()])
        }

        out
    }

    fn name_style(&self) -> Text {
        Text::new(self.name.as_str().into())
    }
}

trait CompletedExt {
    fn spc_style(&self) -> Text;
    fn spc_points(&self) -> u8;
    fn tst_style(&self) -> Text;
    fn tst_points(&self) -> u8;
}

impl CompletedExt for Completed {
    fn spc_style(&self) -> Text {
        let color = match self.spc_points() {
            0 => Color::Red,
            1 | 2 => Color::Yellow,
            3 => Color::Green,
            _ => unreachable!(),
        };
        Text::new(format!("{:.1}", self.spc * 100.0))
            .color(color)
    }

    fn spc_points(&self) -> u8 {
        if self.spc >= 1.0 {
            3
        } else if self.spc >= 0.7 {
            2
        } else if self.spc >= 0.4 {
            1
        } else {
            0
        }
    }

    fn tst_style(&self) -> Text {
        let color = match self.tst_points() {
            0 => Color::Red,
            1 => Color::Yellow,
            2 => Color::Green,
            _ => unreachable!(),
        };
        Text::new(format!("{:.1}", self.tst * 100.0))
            .color(color)
    }

    fn tst_points(&self) -> u8 {
        if self.tst >= 1.0 {
            2
        } else if self.tst >= 0.5 {
            1
        } else {
            0
        }
    }
}

#[test]
fn test_flags_str() {
    let mut flags = Flags::default();
    macro_rules! from_str { ($f:expr) => {{
        expect!(Flags::from_str($f))
    }}}
    assert_eq!(flags, from_str!(""));
    assert_eq!(flags, from_str!("NP"));
    assert_eq!(flags, from_str!("N,parts"));
    assert_eq!(flags, from_str!("name,parts"));
    assert_eq!(flags, from_str!("AFOCT"));
    flags.text = true;
    assert_eq!(flags, from_str!("NTP"));
    assert_eq!(flags, from_str!("TNP"));
    assert_eq!(flags, from_str!("text,parts,name"));
    flags.parts = false;
    flags.text = false;
    assert_eq!(flags, from_str!("N"));
    assert_eq!(flags, from_str!("name"));
}

#[test]
fn test_style() {
    macro_rules! t { [$t:expr] => {{
        Text::new($t.into())
    }}}

    {
        let completed = Completed {
            spc: 0.33435234,
            tst: 1.0,
        };
        assert_eq!(t!("33.4").color(Color::Red), completed.spc_style());
        assert_eq!(t!("100.0").color(Color::Green), completed.tst_style());
    }

    {
        let completed = Completed {
            spc: 0.05,
            tst: 0.0,
        };
        assert_eq!(t!("5.0").color(Color::Red), completed.spc_style());
        assert_eq!(t!("0.0").color(Color::Red), completed.tst_style());
    }

    let art = Artifact {
        id: HashIm::default(),
        name: name!("REQ-foo"),
        file: PathArc::new("/fake"),
        partof: orderset!{},
        parts: orderset!{},
        completed: Completed {
            spc: 1.0,
            tst: 0.003343,
        },
        text: "some text".into(),
        impl_: Impl::NotImpl,
        subnames: orderset!{},
    };
    let expected = vec![
        vec![
            t!("100.0").color(Color::Green),
            t!(" "),
            t!("0.3").color(Color::Red),
        ],
        vec![t!("REQ-foo")],
    ];
    let flags = Flags::default();
    assert_eq!(expected, art.line_style(&flags, false));
}

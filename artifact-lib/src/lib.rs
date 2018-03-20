extern crate base64;
#[macro_use]
extern crate expect_macro;
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate ergo_std;
extern crate ergo_config;
#[macro_use]
extern crate indexmap;
extern crate path_abs;
extern crate siphasher;

#[macro_use]
mod name;
mod dev_prelude;
#[macro_use]
mod family;
#[macro_use]
mod expand_names;
pub mod lint;
pub mod expected;

use std::fmt;
use siphasher::sip128::{Hasher128, SipHasher};

use dev_prelude::*;
pub use name::{Name, SubName, InternalSubName, Type, parse_subnames, NAME_VALID_STR};
pub use family::{Names, auto_partofs};
pub use expand_names::expand_names;
pub use lint::{Categorized, Category, Level, Lint};


#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// #SPC-read-structs.artifact
/// The primary data structure of this library which encapsulates a majority of the useful
/// end product of a user's project.
pub struct Artifact {
    /// The hahs-id of this artifact. This is required for modifying artifacts.
    pub id: HashIm,
    /// The name of the artifact.
    ///
    /// While this library uses `Name` as the key, other libraries (like a web-ui)
    /// might not. This also makes it much simpler to reserialize artifacts as
    /// the mapping cannot be broken.
    pub name: Name,
    /// The file the artifact is defined in.
    pub file: PathArc,
    /// The user defined and calculated `partof` the artifact.
    pub partof: IndexSet<Name>,
    /// The (calculated) parts of the artifact (opposite of partof)
    pub parts: IndexSet<Name>,
    /// The (calculated) completion+tested ratios of the artifact.
    pub completed: Completed,
    /// The user defined text
    pub text: String,
    /// Whether the artifact is implemented directly (in code or `done` field)
    pub impl_: Impl,
    /// Subnames in the text.
    pub subnames: IndexSet<SubName>,
}

impl Artifact {
    pub fn sort(&mut self) {
        self.partof.sort();
        self.parts.sort();
        if let Impl::Code(ref mut c) = self.impl_ {
            c.secondary.sort_keys();
        }
        self.subnames.sort();
    }
}

// ----- IMPL -----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag="type", content="value")]
/// Encapsulates the implementation state of the artifact
pub enum Impl {
    /// The artifact is "defined as done"
    Done(String),
    /// The artifact is at least partially implemented in code.
    Code(ImplCode),
    /// The artifact is not implemented directly at all
    NotImpl,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Encapsulates the implementation state of the artifact in code.
pub struct ImplCode {
    pub primary: Option<CodeLoc>,
    pub secondary: IndexMap<SubName, CodeLoc>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// The location of an artifact reference in code.
pub struct CodeLoc {
    pub file: PathFile,
    pub line: u64,
}

impl CodeLoc {
    pub fn new(file: &PathFile, line: u64) -> CodeLoc {
        CodeLoc {
            file: file.clone(),
            line: line,
        }
    }
}

impl Impl {
    /// Return the `(count, value, secondary_count, secondary_value)`
    /// that this impl should contribute to the "implemented" statistics.
    ///
    /// "secondary" is used because the Done field actually does contribute to
    /// both spc AND tst for REQ and SPC types.
    ///
    /// `subnames` should contain the subnames that exist in that artifact's text
    pub fn to_statistics(&self, subnames: &IndexSet<SubName>) -> (usize, f64, usize, f64) {
        match *self {
            Impl::Done(_) => (1, 1.0, 1, 1.0),
            Impl::Code(ref impl_) => {
                let mut count = 1;
                let mut value = f64::from(impl_.primary.is_some() as u8);
                for sub in subnames.iter() {
                    count += 1;
                    // add 1 if the subname is implemented, else 0
                    value += f64::from(impl_.secondary.contains_key(sub) as u8);
                }
                (count, value, 0, 0.0)
            }
            Impl::NotImpl => {
                if !subnames.is_empty() {
                    // If subnames are defined not being implemented
                    // in code means that you get counts against you
                    (1 + subnames.len(), 0.0, 0, 0.0)
                } else {
                    (0, 0.0, 0, 0.0)
                }
            }
        }
    }

    /// Return whether this is the `Done` variant.
    pub fn is_done(&self) -> bool {
        match *self {
            Impl::Done(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Impl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Impl::Done(ref s) => write!(f, "{}", s),
            Impl::Code(ref c) => write!(f, "{}", c),
            Impl::NotImpl => write!(f, "not directly implemented"),
        }
    }
}

impl fmt::Display for ImplCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref loc) = self.primary {
            write!(f, "{:?}", loc)?;
        }
        if !self.secondary.is_empty() {
            write!(f, "Secondary{:?}", self.secondary)?;
        }
        Ok(())
    }
}

impl fmt::Debug for CodeLoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}[{}]", self.file.display(), self.line)
    }
}


// ----- INTERMEDIATE -----

/// The type used for unique hash ids
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashIm(pub [u8; 16]);

#[derive(Debug, Serialize, Deserialize)]
/// #SPC-structs.artifact_im
pub struct ArtifactIm {
    pub name: Name,
    pub file: PathArc,
    pub partof: IndexSet<Name>,
    pub done: Option<String>,
    pub text: String,
}

// IMPL ArtifactIm

impl ArtifactIm {
    /// Create the intermediate hash.
    ///
    /// This is the primary key used to ensure consistency when modifying artifats via an API.
    pub fn hash_im(&self) -> HashIm {
        let mut hasher = SipHasher::new();
        self.hash(&mut hasher);
        HashIm(hasher.finish128().as_bytes())
    }

    /// Process the `ArtifactIm`.
    ///
    /// This is required whenever serializing/deserializing the ArtifactIm.
    pub fn clean(&mut self) {
        strip_auto_partofs(&self.name, &mut self.partof);
        self.partof.sort();
        clean_text(&mut self.text);
    }
}

impl From<Artifact> for ArtifactIm {
    /// Get an `ArtifactIm` from an `Artifact`
    fn from(art: Artifact) -> ArtifactIm {
        let mut out = ArtifactIm {
            name: art.name,
            file: art.file.into(),
            partof: art.partof,
            done: match art.impl_ {
                Impl::Done(d) => Some(d),
                _ => None,
            },
            text: art.text,
        };
        out.clean();
        out
    }
}

impl Hash for ArtifactIm {
    /// Normal hash **except** we use `name.as_str().hash()` instead of
    /// `name.hash()` to record whether the raw name itself changed.
    ///
    /// Note: normally name is hashed by its type and key.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.as_str().hash(state);
        self.file.hash(state);
        // note: guaranteed that it is always stripped and sorted
        for p in self.partof.iter() {
            p.hash(state);
        }
        self.done.hash(state);
        self.text.hash(state);
    }
}

impl fmt::Display for HashIm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", expect!(json::to_string(&self)))
    }
}

impl fmt::Debug for HashIm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Default for HashIm {
    fn default() -> HashIm {
        HashIm([0; 16])
    }
}

impl Serialize for HashIm {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let b64 = base64::encode_config(&self.0, base64::URL_SAFE_NO_PAD);
        serializer.serialize_str(&b64)
    }
}

impl<'de> Deserialize<'de> for HashIm {
    fn deserialize<D>(deserializer: D) -> result::Result<HashIm, D::Error>
    where
        D: Deserializer<'de>,
    {
        let b64 = String::deserialize(deserializer)?;
        let mut hash = [0_u8; 16];
        base64::decode_config_slice(&b64, base64::URL_SAFE_NO_PAD, &mut hash)
            .map_err(serde::de::Error::custom)?;
        Ok(HashIm(hash))
    }
}

// ------ OPERATIONS -----

#[derive(Debug, Serialize, Deserialize)]
/// #SPC-structs.artifact_op
/// Used for specifying operations to perform.
pub enum ArtifactOp {
    Create {
        artifact: ArtifactIm,
    },
    Update {
        artifact: ArtifactIm,
        orig_id: HashIm,
    },
    Delete {
        name: Name,
        orig_id: HashIm,
    },
}

pub struct IdPieces {
    pub name: Name,
    pub orig_id: Option<HashIm>,
    pub new_id: Option<HashIm>,
}

impl ArtifactOp {
    pub fn clean(&mut self) {
        match *self {
            ArtifactOp::Create { ref mut artifact }
            | ArtifactOp::Update {
                ref mut artifact, ..
            } => artifact.clean(),
            _ => {}
        }
    }

    pub fn id_pieces(&self) -> IdPieces {
        match *self {
            ArtifactOp::Create { ref artifact } => IdPieces {
                name: artifact.name.clone(),
                orig_id: None,
                new_id: Some(artifact.hash_im()),
            },
            ArtifactOp::Update {
                ref artifact,
                ref orig_id,
            } => IdPieces {
                name: artifact.name.clone(),
                orig_id: Some(*orig_id),
                new_id: Some(artifact.hash_im()),
            },
            ArtifactOp::Delete {
                ref name,
                ref orig_id,
            } => IdPieces {
                name: name.clone(),
                orig_id: Some(*orig_id),
                new_id: None,
            },
        }
    }
}

// ----- SETTINGS -----

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
/// Paths that have have be recursively loaded.
pub struct ProjectPaths {
    pub base: PathDir,
    pub code_paths: IndexSet<PathAbs>,
    pub exclude_code_paths: IndexSet<PathAbs>,
    pub artifact_paths: IndexSet<PathAbs>,
    pub exclude_artifact_paths: IndexSet<PathAbs>,
}

// ------ COMPLETED ------

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Copy, Serialize, Deserialize)]
/// #SPC-read-structures.completed
pub struct Completed {
    /// The specification completion ratio.
    pub spc: f32,
    /// The tested completion ratio.
    pub tst: f32,
}


// ------ PROJECT ------

#[derive(Debug, PartialEq, Serialize)]
pub struct Project {
    pub paths: ProjectPaths,
    pub code_impls: IndexMap<Name, ImplCode>,
    pub artifacts: IndexMap<Name, Artifact>,
}

impl Project {
    /// Recursively sort all the items in the project.
    pub fn sort(&mut self) {
        self.code_impls.sort_keys();
        for (_, code) in self.code_impls.iter_mut() {
            code.secondary.sort_keys();
        }
        self.artifacts.sort_keys();
        for (_, art) in self.artifacts.iter_mut() {
            art.sort();
        }
    }
}

// ------ HELPERS ------

/// "clean" the text so that it can be serialized/deserialized to/from any of the supported
/// formats.
pub fn clean_text(s: &mut String) {
    string_trim_right(s);
    if s.contains('\n') {
        s.push('\n');
    }
}


/// Strip the automatic family from the `partof` set.
pub fn strip_auto_partofs(name: &Name, names: &mut IndexSet<Name>) {
    if let Some(p) = name.parent() {
        names.remove(&p);
    }
    if let Some(p) = name.auto_partof() {
        names.remove(&p);
    }
}


/// Inplace trim is annoyingly not in the stdlib
pub fn string_trim_right(s: &mut String) {
    let end = s.trim_right().len();
    s.truncate(end);
}

/// Join a path to an absolute path. Panic if it doesn't exist.
pub fn join_abs<P: AsRef<Path>>(path: &PathAbs, end: P) -> PathFile {
    PathFile::new(path.join(&end)).expect(&format!(
        "{} + {}",
        path.display(),
        end.as_ref().display()
    ))
}


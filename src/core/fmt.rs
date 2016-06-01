use std::fmt::Write;
use std::iter::FromIterator;
use std::path;
use std::collections::HashSet;

use core::types::*;

/// format ArtNames in a reasonable way
pub fn names(names: &Vec<&ArtName>) -> String {
    if names.len() == 0 {
        return "".to_string();
    }
    let mut s = String::new();
    for n in names {
        write!(s, "{}, ", n.raw).unwrap();
    }
    let len = s.len();
    s.truncate(len - 2); // remove last ", "
    s
}

/// settings for what to format
#[derive(Debug, Default, PartialEq, Eq)]
pub struct FmtSettings {
    pub long: bool,
    pub recurse: u8,
    pub path: bool,
    pub parts: bool,
    pub partof: bool,
    pub loc_name: bool,
    pub loc_path: bool,
    pub text: bool,
    pub refs: bool,
    // completed: bool,
    // tested: bool,
}

/// structure which contains all the information necessary to
/// format an artifact for cmdline, html, or anything else
/// purposely doesn't contain items that are *always* displayed
/// such as completed or tested
#[derive(Debug, Default)]
pub struct FmtArtifact {
    pub long: bool,
    pub path: Option<path::PathBuf>,
    pub parts: Option<Vec<FmtArtifact>>,
    pub partof: Option<Vec<ArtName>>,
    pub loc_name: Option<ArtName>,
    pub loc_path: Option<path::PathBuf>,
    pub loc_valid: Option<bool>,
    pub refs: Option<Vec<String>>,
    pub text: Option<String>,
    pub name: ArtName,
}

/// use several configuration options and pieces of data to represent
/// how the artifact should be formatted
pub fn fmt_artifact(name: &ArtName, artifacts: &Artifacts, fmtset: &FmtSettings,
                    recurse: u8, displayed: &mut HashSet<ArtName>) -> FmtArtifact {
    let artifact = artifacts.get(name).unwrap();
    let mut out = FmtArtifact::default();
    if fmtset.path {
        out.path = Some(artifact.path.clone());
    }
    if fmtset.parts {
        let mut parts: Vec<FmtArtifact> = Vec::new();
        for p in &artifact.parts {
            let mut part;
            if recurse == 0 || displayed.contains(&p) {
                part = FmtArtifact::default();
                part.name = p.clone();
            } else {
                part = fmt_artifact(&p, artifacts, fmtset, recurse - 1, displayed);
                displayed.insert(p.clone());
            }
            parts.push(part);
        }
        parts.sort_by_key(|p| p.name.clone());  // TODO: get around clone here
        out.parts = Some(parts);
    }
    if fmtset.partof {
        let mut partof = artifact.partof.iter().map(|p| p.clone()).collect::<Vec<ArtName>>();
        partof.sort();
        out.partof = Some(partof);
    }
    if fmtset.loc_name {
        out.loc_name = match &artifact.loc {
            &Some(ref l) => Some(l.loc.clone()),
            &None => None,
        };
    }
    if fmtset.loc_path {
        out.loc_path = match &artifact.loc {
            &Some(ref l) => {
                if l.path == path::Path::new("") {
                    None
                } else {
                    Some(l.path.clone())
                }
            }
            &None => None,
        }
    }
    if fmtset.refs {
        out.refs = Some(artifact.refs.clone());
    }
    if fmtset.text {
        if fmtset.long {
            out.text = Some(artifact.text.clone());
        } else {
            // return only the first "line" according to markdown
            let mut s = String::new();
            for l in artifact.text.lines() {
                let l = l.trim();
                if l == "" {
                    break;
                }
                s.write_str(l).unwrap();
                s.push(' ');
            }
            out.text = Some(s);
        }
    }
    out.name = name.clone();
    out
}


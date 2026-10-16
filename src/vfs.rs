use crate::PathBuf;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct VfsOverlay {
    pub version: u32,
    #[serde(with = "string_bool")]
    pub case_sensitive: bool,
    #[serde(with = "string_bool")]
    pub overlay_relative: bool,
    pub root_relative: &'static str,
    pub roots: Vec<VfsEntry>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type")]
pub enum VfsEntry {
    File {
        #[serde(with = "pathbuf_serializer")]
        name: PathBuf,
        #[serde(with = "pathbuf_serializer")]
        #[serde(rename = "external-contents")]
        external_contents: PathBuf,
    },
    #[serde(rename = "directory-remap")]
    DirectoryRemap {
        #[serde(with = "pathbuf_serializer")]
        name: PathBuf,
        #[serde(with = "pathbuf_serializer")]
        #[serde(rename = "external-contents")]
        external_contents: PathBuf,
    },
}

mod pathbuf_serializer {
    use crate::PathBuf;
    use serde::{self, Serializer};

    pub fn serialize<S>(value: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.as_str().replace('\\', "/"))
    }
}

impl VfsEntry {
    fn sort_key(&self) -> (&PathBuf, &PathBuf) {
        match self {
            Self::File {
                name,
                external_contents,
            }
            | Self::DirectoryRemap {
                name,
                external_contents,
            } => (name, external_contents),
        }
    }
}

impl VfsOverlay {
    /// Entries are pushed from parallel splat tasks and from `HashMap`
    /// iteration, so their order varies between runs. Sorting keeps the
    /// generated file reproducible, and deduping drops the aliases that several
    /// architectures register for the same path.
    pub fn sort_and_dedup(&mut self) {
        self.roots.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        self.roots.dedup();
    }

    pub fn new() -> Self {
        Self {
            version: 0,
            // By setting `case_sensitive` to false, we're instructing llvm to
            // ignore case when looking at filenames in the VFS, behaving like
            // windows would on a case insensitive filesystem, regardless of the
            // filesystem the toolchain is installed in.
            case_sensitive: false,
            // Treat all the filepaths and directories in the overlay as being
            // relative to the directory this YAML file is in. This makes the
            // overlay portable.
            overlay_relative: true,
            // Resolve the roots as relative to the directory this YAML file is
            // in.
            root_relative: "overlay-dir",
            roots: Vec::new(),
        }
    }
}

impl Default for VfsOverlay {
    fn default() -> Self {
        Self::new()
    }
}

mod string_bool {
    use serde::{self, Serializer};

    pub fn serialize<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string().to_lowercase())
    }
}

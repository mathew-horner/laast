pub mod examples;
pub mod similarity;

use std::collections::HashMap;
use std::path::Path;

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

/// Language-agnostic syntax tree.
pub struct Laast {
    language: Language,
    root: Node,
    hash: String,
}

impl Laast {
    pub fn parse(language: Language, code: &str) -> Result<Self> {
        let hash = <Sha256 as Digest>::digest(code.as_bytes());
        let hash = std::str::from_utf8(&hash)?.to_string();

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(language.ts())?;
        let tree = parser
            .parse(code, None)
            .ok_or_else(|| anyhow!("failed to parse tree sitter tree"))?;

        Ok(Self {
            language,
            // NOCHECKIN: Convert `tree` to Node.
            root: Node::default(),
            hash,
        })
    }
}

/// One node in a language-agnostic syntax tree.
#[derive(Default)]
pub struct Node {
    properties: HashMap<String, String>,
    children: Vec<Box<Node>>,
}

/// All of the supported languages.
pub enum Language {
    CSharp,
    Go,
    Java,
    Javascript,
    Python,
    Ruby,
    Rust,
}

impl Language {
    /// Attempts to infer which language a given file is by inspecting its extension.
    pub fn infer_from_filename(file_name: &Path) -> Result<Self> {
        let extension = file_name
            .extension()
            .ok_or_else(|| anyhow!("file name has no extension"))?;

        let extension = extension
            .to_str()
            .ok_or_else(|| anyhow!("failed to parse file extension"))?;

        match extension {
            "cs" => Ok(Self::CSharp),
            "go" => Ok(Self::Go),
            "java" => Ok(Self::Java),
            "js" => Ok(Self::Javascript),
            "py" => Ok(Self::Python),
            "rb" => Ok(Self::Ruby),
            "rs" => Ok(Self::Rust),
            other => Err(anyhow!("unrecognized extension: {other}")),
        }
    }

    /// Return the associated tree-sitter language.
    #[rustfmt::skip]
    pub fn ts(&self) -> tree_sitter::Language {
        match self {
            Self::CSharp     => tree_sitter_c_sharp::language(),
            Self::Go         => tree_sitter_go::language(),
            Self::Java       => tree_sitter_java::language(),
            Self::Javascript => tree_sitter_javascript::language(),
            Self::Python     => tree_sitter_python::language(),
            Self::Ruby       => tree_sitter_ruby::language(),
            Self::Rust       => tree_sitter_rust::language(),
        }
    }
}

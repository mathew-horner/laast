pub mod examples;
pub mod similarity;

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};

lazy_static! {
    static ref BLACKLISTED_NODE_TYPES: HashSet<&'static str> = maplit::hashset! {
        // Punctuation varies heavily amongst programming languages and can
        // artificially deflate similarity across languages.
        "(", ")", ".", ";", "!", "[", "]", "{", "}", "\"", "\\", ":"
    };
}

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

        let root = Node::from_ts_node(tree.root_node())
            .ok_or_else(|| anyhow!("failed to convert tree sitter tree"))?;

        Ok(Self {
            language,
            root,
            hash,
        })
    }
}

/// One node in a language-agnostic syntax tree.
pub struct Node {
    properties: HashMap<String, String>,
    children: Vec<Box<Node>>,
}

impl Node {
    fn init(ty: impl ToString) -> Self {
        Self {
            properties: maplit::hashmap! { "type".to_string() => ty.to_string() },
            children: Vec::new(),
        }
    }

    fn from_ts_node(ts_node: tree_sitter::Node) -> Option<Self> {
        // In some cases, I've seen this type be a single space character.
        // No matter the situation, we don't care about whitespace, so a strip
        // here suffices.
        let ty = ts_node.kind().trim();

        if BLACKLISTED_NODE_TYPES.contains(ty) {
            return None;
        }

        let mut node = Self::init(ty);
        // NOCHECKIN: Traverse children.
        Some(node)
    }
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

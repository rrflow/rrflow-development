//! One extension-to-language table for every engine projection.
//!
//! The canonical id is what evidence, grammars and records name; the display
//! name is for the project map. Detection is by final extension only; the
//! richer classification (vendored, generated, documentation, data) is P2 of
//! the D-02 evidence-driven attunement plan and will replace this table's
//! role as the only signal, not its ids.

/// A language known to the engine: its canonical id and display name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceLanguage {
    pub id: &'static str,
    pub display: &'static str,
}

/// Detect a language from a path's final extension, case-insensitively.
pub(crate) fn source_language(relative: &str) -> Option<SourceLanguage> {
    let extension = relative.rsplit_once('.')?.1.to_ascii_lowercase();
    let (id, display) = match extension.as_str() {
        "rs" => ("rust", "Rust"),
        "ts" | "tsx" | "mts" | "cts" => ("typescript", "TypeScript"),
        "js" | "jsx" | "mjs" | "cjs" => ("javascript", "JavaScript"),
        "py" | "pyi" => ("python", "Python"),
        "go" => ("go", "Go"),
        "java" => ("java", "Java"),
        "kt" | "kts" => ("kotlin", "Kotlin"),
        "swift" => ("swift", "Swift"),
        "c" | "h" => ("c", "C"),
        "cc" | "cpp" | "cxx" | "hpp" | "hh" => ("cpp", "C++"),
        "cs" => ("csharp", "C#"),
        "rb" => ("ruby", "Ruby"),
        "php" => ("php", "PHP"),
        "dart" => ("dart", "Dart"),
        "lua" => ("lua", "Lua"),
        "md" | "mdx" => ("markdown", "Markdown"),
        "json" => ("json", "JSON"),
        "toml" => ("toml", "TOML"),
        "yaml" | "yml" => ("yaml", "YAML"),
        "html" | "htm" => ("html", "HTML"),
        "css" | "scss" | "sass" | "less" => ("css", "CSS"),
        "vue" => ("vue", "Vue"),
        "svelte" => ("svelte", "Svelte"),
        "sh" | "bash" | "zsh" => ("shell", "Shell"),
        "ps1" | "psm1" => ("powershell", "PowerShell"),
        "sql" => ("sql", "SQL"),
        "ipynb" => ("jupyter", "Jupyter"),
        _ => return None,
    };
    Some(SourceLanguage { id, display })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_canonical_and_match_the_bound_grammars() {
        for path in ["a/b.RS", "x.py", "y.go", "z.cpp", "n.cs", "w.ipynb"] {
            let language = source_language(path).unwrap();
            rrd_contract::CanonicalId::new(language.id).unwrap();
        }
        for language in rrd_attunement::parse::SUPPORTED_LANGUAGES {
            let extension = match language {
                "go" => "f.go",
                "python" => "f.py",
                "rust" => "f.rs",
                other => panic!("unmapped grammar {other}"),
            };
            assert_eq!(source_language(extension).unwrap().id, language);
        }
        assert!(source_language("Makefile").is_none());
        assert!(source_language("archive.tar.gz").is_none());
    }
}

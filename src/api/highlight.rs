use std::{collections::HashMap, fs, iter::Peekable, path::Path};

use anyhow::anyhow;
use regex::{Matches, Regex};
use serde::Deserialize;

/// Single set of regex rules
#[derive(Deserialize)]
struct Language {
    /// file extentions
    extentions: Vec<String>,

    /// Regex rules
    #[serde(flatten, with = "tuple_vec_map")]
    rules: Vec<(String, String)>,
}

/// Set of highlighter
#[derive(Deserialize)]
pub(crate) struct Languages(HashMap<String, Language>);

/// Highlighting range
#[derive(Debug)]
pub(crate) struct HighlightRange {
    /// Part of the code
    text: String,

    /// Style to use
    style: String,
}

impl Languages {
    /// Parse a language set from a string
    pub(crate) fn from_str(string: &str) -> Result<Self, anyhow::Error> {
        toml::from_str::<Self>(string).map_err(|x| x.into())
    }

    /// Load all languages
    pub(crate) fn load(path: &impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let mut languages = HashMap::new();

        // load the included languages
        let included = include_str!("languages.toml");
        languages.extend(Self::from_str(included)?.0.into_iter());

        // load the on-disk languages
        if let Ok(dir) = fs::read_dir(path) {
            for file in dir {
                let text = fs::read_to_string(file?.path())?;
                languages.extend(Self::from_str(&text)?.0.into_iter());
            }
        }

        Ok(Languages(languages))
    }

    /// Highlight one language, from the language name or extention, if it exists
    pub(crate) fn highlight(
        &self,
        code: &str,
        language: &str,
    ) -> Result<Vec<HighlightRange>, anyhow::Error> {
        // find the language
        let language = self
            .0
            .get(language)
            .or_else(|| {
                self.0.iter().find_map(|(_, x)| {
                    if x.extentions
                        .iter()
                        .find(|x| x.as_str() == language)
                        .is_some()
                    {
                        Some(x)
                    } else {
                        None
                    }
                })
            })
            .ok_or(anyhow!("Could not find language {language}"))?;

        // highlight
        let mut rules: Vec<Regex> = Vec::with_capacity(language.rules.len());
        for (_, rule) in language.rules.iter() {
            rules.push(Regex::new(&rule)?);
        }

        let mut matches: Vec<Peekable<Matches>> =
            rules.iter().map(|x| x.find_iter(code).peekable()).collect();

        // highlight the code
        let mut cur_chunk = String::new();
        let mut cur_style = None;
        let mut highlights = Vec::new();

        for (start, character) in code.char_indices() {
            // advance all styles no longer in the range
            for range in matches.iter_mut() {
                if range.peek().map(|x| start >= x.end()).unwrap_or(false) {
                    range.next();
                }
            }

            // find the index of the last style that matches
            let style = matches.iter_mut().enumerate().rev().find_map(|(i, x)| {
                x.peek()
                    .and_then(|x| if start < x.end() { Some(i) } else { None })
            });

            // if they are different, push the current string
            if cur_style != style {
                // only add the chunk if it was highlighted
                if !cur_chunk.is_empty() {
                    highlights.push(HighlightRange {
                        text: cur_chunk,
                        style: cur_style
                            .and_then(|i| language.rules.get(i).map(|x| x.0.clone()))
                            .unwrap_or(String::new()),
                    });
                }

                // reset the rest
                cur_chunk = String::from(character);
                cur_style = style;
            } else {
                // otherwise, push character
                cur_chunk.push(character);
            }
        }

        // push the last, if any
        if !cur_chunk.is_empty() {
            highlights.push(HighlightRange {
                text: cur_chunk,
                style: cur_style
                    .and_then(|i| language.rules.get(i).map(|x| x.0.clone()))
                    .unwrap_or(String::new()),
            });
        }

        Ok(highlights)
    }

    /// Highlight one language as html, if it exists
    pub(crate) fn highlight_html(
        &self,
        code: &str,
        language: &str,
        class_prefix: Option<String>,
    ) -> Result<String, anyhow::Error> {
        Ok(self
            .highlight(code, language)?
            .into_iter()
            .map(|x| {
                format!(
                    r#"<span class="{}{}">{}</span>"#,
                    class_prefix.as_ref().unwrap_or(&String::new()),
                    escape_html(&x.style),
                    escape_html(&x.text)
                )
            })
            .collect())
    }
}

fn escape_html(string: &str) -> String {
    string
        .replace("&", "&amp;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}
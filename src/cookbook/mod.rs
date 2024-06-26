/// Cookbook entry
pub(crate) struct Entry {
    /// name of the entry
    pub(crate) name: &'static str,

    /// Short description
    pub(crate) description: &'static str,

    /// elaborate description
    pub(crate) tutorial: String,
}

pub(crate) fn entries() -> Vec<Entry> {
    vec![Entry {
        name: "markdown",
        description: "Custom markdown rendering",
        tutorial: format!("By default, markdown is rendered straight to HTML. The following script allows custom rendering of markdown\n```lua{}```", include_str!("../../example/scripts/markdown.lua")),
    },
    Entry {
        name: "bibliography",
        description: "Add citations with a bibtex bibliography",
        tutorial: format!("TODO ```lua{}```", include_str!("../../example/scripts/bibliography.lua")),   }]
}

/// All cookbook entries
pub(crate) fn lookup(name: &str) -> Option<Entry> {
    entries().into_iter().find(|x| x.name == name)
}

use std::{
    env::{current_dir, set_current_dir},
    fs::read_to_string,
    path::{Path, PathBuf},
};

use anyhow::anyhow;

use mlua::Lua;

use crate::api::globals::load_globals;
use crate::api::page::Page;
use crate::api::script::Script;
use crate::api::{directory::Directory, styles::load_styles};

/// Entire generated site
pub(crate) struct Site {
    /// the root page
    pub(crate) page: Page,

    /// All warnings generated by the site
    pub(crate) warnings: Vec<String>,

    /// Path the site was generated at
    pub(crate) path: PathBuf,
}

impl Site {
    /// Generate the site
    pub(crate) fn generate(path: Option<PathBuf>) -> Result<Self, anyhow::Error> {
        // path to load from
        let path = path
            .map(|x| Ok(x) as Result<PathBuf, anyhow::Error>)
            .unwrap_or_else(|| {
                let dir = current_dir()?;

                // go up to find the dir containing the site.toml file
                Ok(dir
                    .ancestors()
                    .find(|x| x.join("site.toml").exists())
                    .ok_or_else(|| {
                        anyhow!(
                            "No site.toml found in the current directory or ancestors ({:?})",
                            dir
                        )
                    })?
                    .to_path_buf())
            })?
            .join("site.toml");

        // load the config
        let config = read_to_string(&path)?;

        // path of the directory of where the site is
        let path = path
            .parent()
            .ok_or_else(|| anyhow!("site.toml file was not in a folder!"))?;

        // start lua
        let lua = Lua::new();

        // load the config to the global scope

        // load the static files
        let static_files = Directory::load_static(path.join("static/"), &lua)?;

        // load the styles
        let styles = load_styles(&lua, path.join("styles/"))?;

        // load the settings into the lua environment
        // TODO

        // load the globals
        load_globals(&lua)?;

        // load the root script
        // TODO: make this load directly into lua tables
        let script = Script::load(&path.join("site/"), &lua, &static_files, &styles)?;

        // run the script
        let page = script.run()?;

        // get the warnings
        let warnings: Vec<String> = lua.globals().get("debugWarnings")?;

        Ok(Site {
            page,
            warnings,
            path: path.into(),
        })
    }

    /// Write out the site to a given directory, or public/ in the site directory no path is given
    pub(crate) fn write_to_directory<P: AsRef<Path>>(
        &self,
        path: Option<P>,
    ) -> Result<(), anyhow::Error> {
        self.page.write_to_directory(
            path.map(|x| x.as_ref().into())
                .unwrap_or(self.path.join("public/")),
        )
    }
}

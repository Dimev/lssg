use std::{cell::RefCell, fs, path::Path, rc::Rc};

use latex2mathml::{latex_to_mathml, DisplayStyle};
use mlua::{Lua, Value};
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

use super::file::File;

/// Load all program globals into the lua globals
pub(crate) fn load_globals(
    lua: &Lua,
    path: impl AsRef<Path>,
    debug: bool,
) -> Result<Rc<RefCell<Vec<String>>>, anyhow::Error> {
    // create a new file
    let file = lua.create_function(|_, text: String| Ok(File::New(text)))?;

    // escape html
    // TODO let html_escape = lua.create_function(|_, text: String| Ok())?;
    // TODO: maybe do in lib.lua?

    // convert tex math to mathml
    let mathml = lua.create_function(|_, (text, inline): (String, Option<bool>)| {
        latex_to_mathml(
            &text,
            if inline.unwrap_or(false) {
                DisplayStyle::Inline
            } else {
                DisplayStyle::Block
            },
        )
        .map_err(mlua::Error::external)
    })?;

    // syntax highlighting
    // TODO fix
    let path_owned = path.as_ref().to_owned();
    let highlight = lua.create_function(
        move |_, (text, language, class_prefix): (String, String, Option<String>)| {
            // let mut syntax = SyntaxSet::load_defaults_newlines().into_builder();
            /*syntax
                            .add_from_folder(&path_owned.join("highligts"), true)
                            .map_err(mlua::Error::external)?;
            */
            let syntax = SyntaxSet::load_defaults_newlines();

            // can't find syntax, return no highlight
            if let Some(language) = syntax.find_syntax_by_token(&language) {
                let mut generator = ClassedHTMLGenerator::new_with_class_style(
                    language,
                    &syntax,
                    // SYNTECT WHY
                    // this is dumb, why does prefix need to be static
                    // TODO: find some way to reclaim the memory here
                    class_prefix
                        .map(|x| ClassStyle::SpacedPrefixed { prefix: x.leak() })
                        .unwrap_or(ClassStyle::Spaced),
                );
                for line in LinesWithEndings::from(&text) {
                    generator
                        .parse_html_for_line_which_includes_newline(line)
                        .map_err(mlua::Error::external)?;
                }
                Ok(Some(generator.finalize()))
            } else {
                Ok(None)
            }
        },
    )?;

    // warn function
    let warnings = Rc::new(RefCell::new(Vec::<String>::new()));
    let warnings_cloned = warnings.clone();
    lua.set_warning_function(move |lua, text, _| {
        // Get the stack trace
        let mut trace = Vec::new();
        for frame in (1..).map_while(|i| lua.inspect_stack(i)) {
            let name = frame.source().short_src.unwrap_or("?".into());
            let what = frame.names().name_what;
            let func = frame.names().name.unwrap_or("?".into());
            let line = frame.curr_line();
            let line = if line < 0 {
                format!("")
            } else {
                format!(":{}", line)
            };
            if let Some(what) = what {
                trace.push(format!("    {}{}: in {} '{}'", name, line, what, func));
            } else {
                trace.push(format!("    {}{}: in {}", name, line, func));
            }
        }

        // give the stack trace to the warnings
        let warning = format!(
            "runtime warning: {}\nstack traceback:\n{}",
            text,
            trace.join("\n")
        );
        warnings_cloned.borrow_mut().push(warning);
        Ok(())
    });

    // require function
    let path_owned = path.as_ref().to_owned();
    let require = lua.create_function(move |lua, script: String| {
        let path = path_owned.join("scripts").join(&script);
        let code = fs::read_to_string(path).map_err(mlua::Error::external)?;
        let function = lua.load(code).into_function()?;
        lua.load_from_function::<Value>(&format!("scripts/{script}"), function)
    })?;

    // load
    let table = lua.create_table()?;
    table.set("file", file)?;
    table.set("debug", debug)?;
    table.set("highlight", highlight)?;
    table.set("latextomathml", mathml)?;
    lua.globals().set("yassg", table)?;
    lua.globals().set("require", require)?;

    Ok(warnings)
}

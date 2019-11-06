extern crate getopts;
extern crate markdown;
extern crate regex;
use getopts::Options;
mod reader;
use regex::Regex;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    // Parse args
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("s", "source", "Source directory. Defaults to src/", "DIR");
    opts.optopt(
        "o",
        "output",
        "Name of directory you want to write to. Defaults to public/",
        "DIR",
    );
    opts.optopt(
        "t",
        "templates",
        "Templates directory. Defaults to templates/",
        "DIR",
    );
    let a = opts
        .parse(&args[1..])
        .expect("Could not parse command line arguments.");

    if a.opt_present("h") {
        let brief = format!("Usage: {} small-site [options]", program);
        print!("{}", opts.usage(&brief));
        return;
    }

    let a_src = a.opt_str("s").unwrap_or("src".to_string());
    let a_tpl = a.opt_str("t").unwrap_or("templates".to_string());
    let a_dst = a.opt_str("o").unwrap_or("public".to_string());

    let src_dir = Path::new(&a_src);
    let tpl_dir = Path::new(&a_tpl);
    let dst_dir = Path::new(&a_dst);
    // Regex for finding {{foo}} in tpl
    let re = Regex::new(r"\{\{[a-zA-Z][0-9a-zA-Z_]*}}").unwrap();

    // Ignore the error because it most likely will say dir already exists
    let _ = fs::create_dir(&dst_dir);

    let mut tpls = reader::FileReader::new();

    // We go through each file (incl. dirs) in the src dir. If we encounter any errors, we just
    // log it and take the next file.
    for path in read_dir(src_dir) {
        // src/foo/bar --> foo/bar
        let src_rel_path = match path.strip_prefix(src_dir) {
            Ok(p) => p,
            Err(_) => continue,
        };

        if path.is_dir() {
            // Create directory in ./public/
            // Ignore error because it is very likely to exist already.
            let _ = fs::create_dir_all(&dst_dir.join(src_rel_path));
            continue;
        }

        let is_html = has_extension(&src_rel_path, "html");
        let is_markdown = has_extension(&src_rel_path, "md");

        // Ignore any file that isn't html or markdown.
        if !is_html && !is_markdown {
            continue;
        }

        let dst = dst_dir.join(src_rel_path).with_extension("html");
        let result = convert_and_create(&path, &dst, &tpl_dir, is_markdown, &re, &mut tpls);

        if let Err(e) = result {
            eprintln!("{}", e);
        }
    }
}

fn convert_and_create(
    file: &Path,
    dst: &Path,
    tpl_dir: &Path,
    is_markdown: bool,
    re: &Regex,
    tpls: &mut reader::FileReader,
) -> Result<(), String> {
    // Read content of src file
    let content = fs::read_to_string(&file)
        .map_err(|e| format!("Could not read {:?}. Got error: {}", &file, e))?;

    // Parse variables (at the top of the content file)
    let (vars, content) = match split_once(&content, "\n---") {
        Some((h, c)) => (header_to_variables(h), c.to_string()),
        None => (HashMap::new(), content),
    };

    // convert markdown to html if needed
    let content = if is_markdown {
        markdown::to_html(&content)
    } else {
        content
    };

    // Get template
    let tpl_path_rel = vars.get("template").unwrap_or(&"default.html");
    let tpl = tpls
        .get(tpl_dir.join(&tpl_path_rel))
        .as_ref()
        .ok_or_else(|| {
            format!(
                "Could not find template for {:?}. Searched here {:?}",
                &file,
                tpl_dir.join(&tpl_path_rel).display(),
            )
        })?;

    // Merge template and content
    let mut html = String::new();
    let mut at: usize = 0;

    // Look for {{key}}'s in the template and replace them with the values from the markdown
    // file's header variables, where {{content}} means the whole (converted) md file.
    for c in re.captures_iter(&tpl) {
        let m = match c.get(0) {
            Some(m) => m,
            None => continue,
        };

        html += &tpl[at..m.start()];

        // {{key}} -> key
        let key = tpl[m.start()..m.end()]
            .trim_start_matches('{')
            .trim_end_matches('}')
            .trim();

        html += if key == "content" {
            &content
        } else {
            match vars.get(key) {
                Some(value) => value,
                // if we didn't find it in the variables, then we probably shouldn't have tried to
                // replace it, so we just put back whatever our regex found.
                None => &m.as_str(),
            }
        };

        at = m.end();
    }

    html += &tpl[at..];

    // Create and write to file.html
    let mut f = File::create(&dst)
        .map_err(|e| format!("Tried to create {:?}, but got error: {:?}", &dst, e))?;

    f.write_all(html.as_bytes())
        .map_err(|e| format!("Tried to write to {:?}, but got error: {:?}", &dst, e))?;

    Ok(())
}

fn has_extension(p: &Path, e: &str) -> bool {
    Some(e) == p.extension().and_then(OsStr::to_str)
}

// a=b\nc=d --> {"a": "b", "c": "d"}
fn header_to_variables(header: &str) -> HashMap<&str, &str> {
    let mut variables = HashMap::new();

    for line in header.lines() {
        if let Some((k, v)) = split_once(line, "=") {
            let k = k.trim();
            variables.insert(k, v);
        }
    }

    return variables;
}

// split_once("a-b-c", "-") --> ("a", "b-c")
fn split_once<'a>(string: &'a str, splitter: &str) -> Option<(&'a str, &'a str)> {
    let mut splitter = string.splitn(2, splitter);
    let first = splitter.next()?.trim();
    let second = splitter.next()?.trim();
    Some((first, second))
}

fn read_dir(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if !dir.is_dir() {
        return paths;
    }
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return paths,
    };
    for path in entries.filter_map(|e| e.ok()).map(|e| e.path()) {
        if path.is_dir() {
            let mut children = read_dir(&path);
            paths.push(path);
            paths.append(&mut children);
        } else {
            paths.push(path);
        }
    }
    paths
}

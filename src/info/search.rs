use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::{InfoSettings, SearchPath},
    info::{self, MenuItem, TextBlockContent},
};

pub struct ManualPath {
    file: PathBuf,
    start_node: Option<String>,
}

impl ManualPath {
    pub fn file(&self) -> &Path {
        &self.file
    }

    pub fn start_node(&self) -> Option<&str> {
        self.start_node.as_deref()
    }
}

pub fn search_manual(settings: &InfoSettings, query: &str) -> impl Iterator<Item = ManualPath> {
    settings.paths().iter().filter_map(move |p| match p {
        SearchPath::Immediate => search_immediate(&PathBuf::from(query)),
        SearchPath::DirFile(path_buf) => search_dir_file(path_buf, query),
        SearchPath::DirsFromEnv(var) => search_dirs_from_env(var, query),
        SearchPath::Dir(path_buf) => search_dir(path_buf, query),
    })
}

fn search_immediate(query: &Path) -> Option<ManualPath> {
    if let Ok(true) = fs::exists(query) {
        Some(ManualPath {
            file: query.into(),
            start_node: None,
        })
    } else {
        None
    }
}

fn search_dir_file(dir_file: &Path, query: &str) -> Option<ManualPath> {
    let man = info::read_nonsplit_manual(&fs::read_to_string(dir_file).ok()?).ok()?;

    let menu = man
        .nodes()
        .iter()
        .flat_map(|n| n.general_text.iter())
        .find_map(|t| match &t.content {
            TextBlockContent::Menu(menu) => Some(menu),
            _ => None,
        })?;

    menu.items.iter().find_map(|i| match i {
        info::MenuItem::Entry(entry) => {
            if let Some(ref label) = entry.label
                && label.eq_ignore_ascii_case(query)
                && let Some(ref infofile) = entry.id.infofile
            {
                let mut f = search_dir(dir_file.parent()?, infofile)?;
                f.start_node = entry.id.nodename.clone();
                Some(f)
            } else {
                None
            }
        }
        MenuItem::Comment(_) => None,
    })
}

fn search_dirs_from_env(_var: &str, _query: &str) -> Option<ManualPath> {
    None
}

fn search_dir(dir: &Path, query: &str) -> Option<ManualPath> {
    let base = PathBuf::from(query);
    if base.is_absolute() {
        return None;
    }

    let base = dir.join(base);
    if let Some(info) = search_immediate(&base.with_added_extension("info")) {
        return Some(info);
    }
    if let Some(gz) = search_immediate(&base.with_added_extension("info.gz")) {
        return Some(gz);
    }

    None
}

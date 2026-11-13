//! Literal module census, not macro expansion or a transitive call-graph proof.
use std::collections::BTreeSet;
use std::path::Path;
use vectors::keyed_final_manifest::FrozenItem;

#[path = "../../src/keyed_final_manifest/pins.rs"]
mod pins;

fn children(path: &str, source: &str, known: &BTreeSet<&str>) -> Result<(), String> {
    let file = Path::new(path);
    let parent = file.parent().unwrap();
    let stem = file.file_stem().unwrap();
    let directory = if stem == "mod" || stem == "lib" {
        parent.to_path_buf()
    } else {
        parent.join(stem)
    };
    let mut test = false;
    let mut explicit = None;
    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if line == "#[cfg(test)]" {
            test = true;
            continue;
        }
        if line.starts_with("#[path") {
            if !line.ends_with(']') {
                return Err("inline path attribute requires census review".into());
            }
            explicit = Some(
                line.split('"')
                    .nth(1)
                    .ok_or("unrecognized path attribute")?
                    .to_owned(),
            );
            continue;
        }
        if line.starts_with("#[") {
            if line.contains("mod ") {
                return Err("inline attribute requires census review".into());
            }
            continue;
        }
        let words: Vec<_> = line.split_whitespace().collect();
        if let Some(index) = words.iter().position(|word| *word == "mod")
            && !test
        {
            let name = words.get(index + 1).ok_or("missing module name")?;
            if !name.ends_with(';') {
                return Err("inline module requires census review".into());
            }
            let child = explicit.as_ref().map_or_else(
                || directory.join(format!("{}.rs", name.trim_end_matches(';'))),
                |relative| parent.join(relative),
            );
            if !known.contains(child.to_str().unwrap()) {
                return Err(child.display().to_string());
            }
        }
        test = false;
        explicit = None;
    }
    Ok(())
}

/// Check every compiled-in source and pin omission/path-attribute mutants.
pub fn check() {
    let known: BTreeSet<_> = pins::SOURCES.iter().map(|(_, path, _, _)| *path).collect();
    for (_, path, bytes, _) in pins::SOURCES {
        if Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
        {
            assert_eq!(
                children(path, std::str::from_utf8(bytes).unwrap(), &known),
                Ok(()),
                "{path}"
            );
        }
    }
    assert_eq!(pins::SECOND_KEY, vectors::keyed_final_manifest::SECOND_KEY);
    let path = "packages/compiler/src/engine_keyed.rs";
    let control = "pub mod program;";
    assert_eq!(children(path, control, &known), Ok(()));
    let mut missing = known.clone();
    missing.remove("packages/compiler/src/engine_keyed/program.rs");
    assert_eq!(
        children(path, control, &missing),
        Err("packages/compiler/src/engine_keyed/program.rs".into())
    );
    assert_eq!(
        children(
            path,
            "#[path = \"engine_keyed/program.rs\"]\nmod program;",
            &known
        ),
        Ok(())
    );
    assert_eq!(
        children(
            path,
            "#[path = \"engine_keyed/absent.rs\"]\nmod program;",
            &known
        ),
        Err("packages/compiler/src/engine_keyed/absent.rs".into())
    );
    assert_eq!(children(path, "#[cfg(test)]\nmod absent;", &known), Ok(()));
    assert_eq!(
        children(path, "#[path = \"absent.rs\"] mod absent;", &known),
        Err("inline path attribute requires census review".into())
    );
    assert_eq!(
        children(path, "mod absent;", &known),
        Err("packages/compiler/src/engine_keyed/absent.rs".into())
    );
}

use convert_case::{Boundary, Case, Converter};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

/// Helper function to collect files recursively while applying filters
pub(crate) fn collect_files(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    include_regex: &Option<Regex>,
    ignore_regex: &Option<Regex>,
) -> std::io::Result<()> {
    if !dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Directory not found: {}", dir.display()),
        ));
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let path_str = path.to_string_lossy();

        if ignore_regex
            .as_ref()
            .is_some_and(|regex| regex.is_match(&path_str))
        {
            continue;
        }

        if path.is_dir() {
            collect_files(&path, files, include_regex, ignore_regex)?;
        } else {
            if include_regex
                .as_ref()
                .is_none_or(|regex| regex.is_match(&path_str))
            {
                files.push(path);
            }
        }
    }

    Ok(())
}

/// Convert file path to a valid enum variant name in UpperCamelCase
pub(crate) fn path_to_variant_name<P: AsRef<Path>>(path: P) -> String {
    let path_str = path.as_ref().to_string_lossy();

    let conv = Converter::new()
        .add_boundaries(&[
            Boundary::from_delim("/"),
            Boundary::from_delim(r"\"),
            Boundary::from_delim("."),
        ])
        .to_case(Case::Pascal);

    let variant_name = conv.convert(path_str);

    // Try to ensure it's a valid Rust identifier
    if variant_name.starts_with(|first: char| first.is_numeric()) {
        format!("Asset{}", variant_name)
    } else {
        variant_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_file_paths() {
        assert_eq!(path_to_variant_name("image.png"), "ImagePng");
        assert_eq!(path_to_variant_name("style.css"), "StyleCss");
    }

    #[test]
    fn test_nested_paths() {
        assert_eq!(path_to_variant_name("ui/button.svg"), "UiButtonSvg");
        assert_eq!(
            path_to_variant_name("assets/icons/home.png"),
            "AssetsIconsHomePng"
        );
    }

    #[test]
    fn test_windows_path_separators() {
        assert_eq!(path_to_variant_name(r"ui\button.svg"), "UiButtonSvg");
        assert_eq!(
            path_to_variant_name(r"assets\icons\home.png"),
            "AssetsIconsHomePng"
        );
    }

    #[test]
    fn test_paths_with_hyphens() {
        assert_eq!(path_to_variant_name("user-icon.png"), "UserIconPng");
        assert_eq!(
            path_to_variant_name("ui/user-profile/avatar_small.jpg"),
            "UiUserProfileAvatarSmallJpg"
        );
    }

    #[test]
    fn test_paths_with_underscores() {
        assert_eq!(path_to_variant_name("button_large.png"), "ButtonLargePng");
    }

    #[test]
    fn test_paths_starting_with_numbers() {
        assert_eq!(path_to_variant_name("1icon.png"), "Asset1IconPng");
        assert_eq!(path_to_variant_name("2021/logo.png"), "Asset2021LogoPng");
    }

    #[test]
    fn test_paths_with_multiple_dots() {
        assert_eq!(path_to_variant_name("config.dev.json"), "ConfigDevJson");
    }
}

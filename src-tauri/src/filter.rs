use std::path::Path;

const EXACT_EXCLUDED: &[&str] = &[
    "habitat_site_context",
    "habitat_site_context.rb",
    "d5_converter",
    "d5_converter.rb",
    "d5_lightening",
    "d5_lightening.rb",
    "d5_render",
    "d5_render.rb",
    "su_to_d5render",
    "enscape",
    "enscape.rb",
    "enscape_loader",
    "enscape_loader.rb",
    "vfs",
    "vfs.rb",
    "vray",
    "vray.rb",
    "chaos",
    "lumionlivesync",
    "lumionlivesync.rb",
    "lumionlivesyncforsketchup",
    "lumionlivesyncforsketchup.rb",
    "datasmithsketchupexporter",
    "datasmithsketchupexporter.rb",
    "twinmotiondirectlink",
    "twinmotiondirectlink.rb",
    "corona",
    "corona.rb",
    "thea_render",
    "thea_render.rb",
    "thea_for_sketchup",
    "thea_for_sketchup.rb",
];

pub fn is_excluded(path_or_name: &str) -> bool {
    if path_or_name.trim().is_empty() {
        return false;
    }

    // Get the first path segment (root folder or file name)
    let p = Path::new(path_or_name);
    let first_segment = match p.components().next() {
        Some(comp) => comp.as_os_str().to_string_lossy().to_string(),
        None => path_or_name.to_string(),
    };

    let lower = first_segment.to_lowercase();

    // 1. SketchUp built-in prefix: su_*
    if lower.starts_with("su_") {
        return true;
    }

    // 2. Exact match check
    for item in EXACT_EXCLUDED {
        if lower == *item {
            return true;
        }
    }

    // 3. Match without extension
    if let Some(without_ext) = Path::new(&lower).file_stem().and_then(|s| s.to_str()) {
        for item in EXACT_EXCLUDED {
            if without_ext == *item {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exclusions() {
        assert!(is_excluded("su_sandbox.rb"));
        assert!(is_excluded("su_dynamiccomponents/tool.rbe"));
        assert!(is_excluded("habitat_site_context/clear_location.rbe"));
        assert!(is_excluded("d5_converter.rb"));
        assert!(is_excluded("vfs.rb"));
        assert!(is_excluded("enscape/loader.rb"));
        assert!(is_excluded("lumionlivesync.rb"));
        assert!(!is_excluded("curviloft.rb"));
        assert!(!is_excluded("jointpushpull/main.rb"));
    }
}

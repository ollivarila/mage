use std::path::PathBuf;

pub(crate) fn get_full_path<P: Into<PathBuf>>(path: P) -> PathBuf {
    let home = std::env::var("HOME").unwrap();
    let path: PathBuf = path.into();

    if path.starts_with("~") {
        let mut full_path = PathBuf::from(home);
        for item in path.iter() {
            if item.to_str().unwrap() != "~" {
                full_path.push(item)
            }
        }
        return full_path;
    }

    path
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_full_path() {
        let home = std::env::var("HOME").unwrap();
        let expected = PathBuf::from(format!("{home}/test"));
        let path = get_full_path("~/test");
        assert_eq!(path, expected);
    }
}

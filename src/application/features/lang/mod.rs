use std::{collections::HashMap, fs, path::PathBuf};

pub(crate) fn init_lang(path: PathBuf) -> Result<(), std::io::Error> {
    if path.exists() {
        return Ok(());
    }

    fs::write(path, String::from(r#"{"example.lang.here": "Fighting Helicopter!"}"#))?;
    Ok(())
}

pub(crate) fn read_lang(path: PathBuf) -> Result<HashMap<String, String>, std::io::Error> {
    let data_str: String = fs::read_to_string(path)?;
    let data: HashMap<String, String> = serde_json::from_str(&data_str)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(data)
}

pub fn get_translation(data: &HashMap<String, String>, key: &str) -> Option<String> {
    data.get(key).cloned()
}

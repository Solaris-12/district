use rusqlite::Result;

pub trait DatabaseOperations {
    fn setup(db_path: &str) -> Result<Self>
    where
        Self: Sized;
}

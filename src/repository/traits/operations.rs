use crate::error::error_server::ErrorServer;
///
/// Trait that represents the different
/// operations that can be performed
/// when operating our database, known as CRUD operations
///
pub trait Operations<K, T> {
    /// search for a key in the "database"
    fn search(&self, key: K) -> Result<Option<T>, ErrorServer>;
    /// add key and value into the "database"
    fn add(&self, key: K, value: T) -> Result<bool, ErrorServer>;
    /// delete a key in the database
    fn delete(&self, key: K) -> Result<bool, ErrorServer>;
    /// update an existant value
    fn update(&self, key: K, value: T) -> Result<bool, ErrorServer>;
    /// find all the values contained in the "database"
    fn find_all(&self) -> Result<Vec<T>, ErrorServer>;
}

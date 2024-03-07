use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
};

use crate::error::error_server::ErrorServer;
use crate::repository::traits::operations::Operations;

///
/// Struct that implements a key-value pair collection of two generic values
/// as needed by the DAO programming pattern represented
/// with the [`Operations`] trait.
///
#[derive(Debug)]
pub struct HashMapDao<K, T> {
    values: Arc<RwLock<HashMap<K, T>>>,
}

impl<K, T> HashMapDao<K, T> {
    ///
    /// Constructor
    ///
    /// # Returns
    /// Generates an empty HashMap entity and adds a lock on it, in order to be shared
    /// between threads if it's neccesary.
    ///
    ///
    pub fn new() -> Self {
        HashMapDao {
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<K, T> Default for HashMapDao<K, T> {
    fn default() -> Self {
        Self::new()
    }
}

///
/// implementation of the operations trait,
/// for a description of the functions,
/// consult the corresponding documentation
///
///
impl<K, T> Operations<K, T> for HashMapDao<K, T>
where
    K: Eq + PartialEq + Hash,
    T: Clone,
{
    fn search(&self, key: K) -> Result<Option<T>, ErrorServer> {
        let clients = match self.values.read() {
            Ok(c) => c,
            Err(_) => return Err(ErrorServer::PoisonedThread),
        };

        Ok(clients.get(&key).cloned())
    }

    fn add(&self, key: K, value: T) -> Result<bool, ErrorServer> {
        {
            let x = match self.values.read() {
                Ok(c) => c,
                Err(_e) => return Err(ErrorServer::PoisonedThread),
            };
            if x.contains_key(&key) {
                return Ok(false);
            }
        }
        {
            let mut x = match self.values.write() {
                Ok(c) => c,
                Err(_) => return Err(ErrorServer::PoisonedThread),
            };
            x.insert(key, value);
        }

        Ok(true)
    }

    fn delete(&self, key: K) -> Result<bool, ErrorServer> {
        {
            let x = match self.values.read() {
                Ok(c) => c,
                Err(_e) => return Err(ErrorServer::PoisonedThread),
            };
            if !x.contains_key(&key) {
                return Ok(false);
            }
        }
        {
            let mut x = match self.values.write() {
                Ok(c) => c,
                Err(_e) => return Err(ErrorServer::PoisonedThread),
            };

            x.remove(&key);
        }
        Ok(true)
    }

    fn update(&self, key: K, value: T) -> Result<bool, ErrorServer> {
        let mut x = match self.values.write() {
            Ok(c) => c,
            Err(_e) => return Err(ErrorServer::PoisonedThread),
        };

        x.insert(key, value);
        Ok(true)
    }

    fn find_all(&self) -> Result<Vec<T>, ErrorServer> {
        let x = match self.values.write() {
            Ok(c) => c,
            Err(_e) => return Err(ErrorServer::PoisonedThread),
        };

        let mut list = Vec::new();
        for v in x.values() {
            list.push(v.clone());
        }
        Ok(list)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn search_a_non_existant_key() -> Result<(), ErrorServer> {
        let repo: HashMapDao<String, String> = HashMapDao::new();
        let result: Option<_> = repo.search("usuario".to_string())?;
        assert!(result.is_none());
        Ok(())
    }

    #[test]
    fn search_an_existant_key() -> Result<(), ErrorServer> {
        let repo: HashMapDao<String, String> = HashMapDao::new();
        repo.add(String::from("user"), String::from("usuario"))?;
        let result: Option<_> = repo.search("user".to_string())?;
        assert!(result.is_some());
        Ok(())
    }

    #[test]
    fn add_value() -> Result<(), ErrorServer> {
        let repo: HashMapDao<String, String> = HashMapDao::new();
        assert!(repo.add(String::from("user"), String::from("usuario"))?);
        let result = repo.search(String::from("user"))?;
        assert_eq!(result, Some(String::from("usuario")));
        Ok(())
    }

    #[test]
    fn delete_value() -> Result<(), ErrorServer> {
        let repo: HashMapDao<String, String> = HashMapDao::new();
        repo.add(String::from("user"), String::from("usuario"))?;
        assert!(repo.delete(String::from("user"))?);
        let result = repo.search(String::from("user"))?;
        assert!(result.is_none());
        Ok(())
    }
    #[test]
    fn delete_non_existing_value() -> Result<(), ErrorServer> {
        let repo: HashMapDao<String, String> = HashMapDao::new();
        assert!(!repo.delete(String::from("user"))?);
        Ok(())
    }

    #[test]
    fn update_value() -> Result<(), ErrorServer> {
        let repo: HashMapDao<String, String> = HashMapDao::new();
        repo.add(String::from("user"), String::from("test"))?;
        assert!(repo.update(String::from("user"), String::from("Usuario"))?);
        assert_eq!(
            repo.search(String::from("user"))?,
            Some(String::from("Usuario"))
        );
        Ok(())
    }

    #[test]
    fn find_all_values() -> Result<(), ErrorServer> {
        let repo: HashMapDao<String, String> = HashMapDao::new();
        repo.add(String::from("user1"), String::from("test3"))?;
        repo.add(String::from("user2"), String::from("test2"))?;
        repo.add(String::from("user3"), String::from("test1"))?;

        let result = repo.find_all()?;

        let expected = vec![
            String::from("test1"),
            String::from("test2"),
            String::from("test3"),
        ];

        assert!(expected.iter().all(|item| result.contains(item)));
        Ok(())
    }
}

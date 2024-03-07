use super::{
    query::{QueryAnswer, QueryOption},
    traits::operations::Operations,
};
use crate::error::error_server::ErrorServer;
use crate::repository::query::Query;
use crate::repository::repository_channel::{client_channel::ClientChannel, RepoChannel};

///
/// Struct that contains the entity in charge of the communication [`RepoChannel`]
/// between repository's client and the database.
///
/// A database can be any entity that implementes the [`Operations`] trait.
///
pub struct Repository<K, T> {
    repo_channel: RepoChannel<K, T>,
    client_channel: ClientChannel<K, T>,
    dao: Box<dyn Operations<K, T>>,
}

impl<K, T> Repository<K, T> {
    ///
    /// Constructor
    ///
    /// # Arguments
    /// * `dao: Box<dyn Operations<K, T>>` : box with that contains a dyn that
    /// implements the Operations trait
    ///  
    /// # Example
    ///
    /// ```rust
    ///     use irc_project::repository::repo::Repository;
    ///     use irc_project::repository::{dao::hash_map_dao::HashMapDao};
    ///
    ///     let dao: HashMapDao<String, String> = HashMapDao::new();
    ///     let dao_box = Box::new(dao);
    ///     let repo: Repository<String, String> = Repository::new(dao_box);
    /// ```
    ///
    ///
    pub fn new(dao: Box<dyn Operations<K, T>>) -> Self {
        let (repo_channel, client_channel) = RepoChannel::new();
        Repository {
            repo_channel,
            client_channel,
            dao,
        }
    }

    ///
    /// Function that returns a [`ClientChannel`] in charge of
    /// sending and receiving information from the repository.
    ///
    pub fn get_client_channel(&self) -> ClientChannel<K, T> {
        self.client_channel.clone()
    }

    ///
    /// Function that contains the principal loop of
    /// the repository, in charge of handling a received [`Query`] and  
    /// sending the [`QueryAnswer`] through the [`ClientChannel`].
    ///
    pub fn run(&self) -> Result<(), ErrorServer> {
        //loop principal que recibe las instrucciones
        loop {
            let response = match self.repo_channel.try_recv() {
                Ok(c) => self.handle_received_query(c)?,
                Err(_) => continue,
            };
            self.repo_channel.send(response)?;
        }
    }

    fn handle_received_query(&self, query: Query<K, T>) -> Result<QueryAnswer<T>, ErrorServer> {
        match *query.get_option() {
            QueryOption::Search => self.search(query),
            QueryOption::Add => self.add(query),
            QueryOption::Delete => self.delete(query),
            QueryOption::Update => self.update(query),
            QueryOption::FindAll => self.find_all(),
        }
    }

    fn search(&self, query: Query<K, T>) -> Result<QueryAnswer<T>, ErrorServer> {
        let x = match query.get_arguments().0 {
            Some(c) => c,
            None => return Err(ErrorServer::BadQuery),
        };
        let result = self.dao.search(x)?;
        Ok(QueryAnswer::Search(result))
    }

    fn add(&self, query: Query<K, T>) -> Result<QueryAnswer<T>, ErrorServer> {
        let (key, value) = match query.get_arguments() {
            (Some(k), Some(v)) => (k, v),
            _ => return Err(ErrorServer::BadQuery),
        };
        let result = self.dao.add(key, value)?;
        Ok(QueryAnswer::Add(result))
    }

    fn delete(&self, query: Query<K, T>) -> Result<QueryAnswer<T>, ErrorServer> {
        let key = match query.get_arguments().0 {
            Some(c) => c,
            None => return Err(ErrorServer::BadQuery),
        };
        let result = self.dao.delete(key)?;
        Ok(QueryAnswer::Delete(result))
    }

    fn update(&self, query: Query<K, T>) -> Result<QueryAnswer<T>, ErrorServer> {
        let (key, value) = match query.get_arguments() {
            (Some(k), Some(v)) => (k, v),
            _ => return Err(ErrorServer::BadQuery),
        };
        let result = self.dao.update(key, value)?;
        Ok(QueryAnswer::Update(result))
    }

    fn find_all(&self) -> Result<QueryAnswer<T>, ErrorServer> {
        let result = self.dao.find_all()?;
        Ok(QueryAnswer::FindAll(result))
    }
}

#[cfg(test)]
mod test {
    use std::{error::Error, sync::mpsc, thread};

    use crate::repository::dao::hash_map_dao::HashMapDao;

    use super::*;

    fn initilize_mock_repo() -> ClientChannel<String, String> {
        let (x, y) = mpsc::channel::<ClientChannel<String, String>>();
        let sender = x;
        let _x = thread::spawn(move || {
            let dao: HashMapDao<String, String> = HashMapDao::new();
            dao.add("user1".to_string(), "juan".to_string())
                .expect("errror");
            dao.add("user2".to_string(), "marcos".to_string())
                .expect("errror");
            let dao_box = Box::new(dao);
            let repo: Repository<String, String> = Repository::new(dao_box);
            let send1 = repo.get_client_channel();
            sender.send(send1).expect("error");
            repo.run().expect("error");
        });

        y.recv().expect("error")
    }

    #[test]
    fn search_is_ok() -> Result<(), Box<dyn Error>> {
        let ch = initilize_mock_repo();
        let search = ch.search("user1".to_string()).unwrap();
        let expected = "juan".to_string();
        if let Some(result) = search {
            assert_eq!(result, expected);
        }
        Ok(())
    }

    #[test]
    fn search_for_a_non_existant_key() -> Result<(), Box<dyn Error>> {
        let ch = initilize_mock_repo();
        let search = ch.search("user3".to_string()).unwrap();
        assert!(search.is_none());
        Ok(())
    }

    #[test]
    fn add_returns_true_when_ok() -> Result<(), Box<dyn Error>> {
        let ch = initilize_mock_repo();
        let add = ch.add("user3".to_string(), "pedro".to_string()).unwrap();
        assert!(add);
        Ok(())
    }

    #[test]
    fn add_returns_false_for_an_existant_key() -> Result<(), Box<dyn Error>> {
        let ch = initilize_mock_repo();
        let add = ch.add("user1".to_string(), "juan".to_string()).unwrap();
        assert!(!add);
        Ok(())
    }

    #[test]
    fn delete_an_existant_key() {
        let ch = initilize_mock_repo();
        let delete = ch.delete("user1".to_string()).unwrap();
        assert!(delete);
    }

    #[test]
    fn delete_a_non_existant_key() -> Result<(), Box<dyn Error>> {
        let ch = initilize_mock_repo();
        let delete = ch.delete("user3".to_string()).unwrap();
        assert!(!delete);
        Ok(())
    }

    #[test]
    fn update_an_existant_key() -> Result<(), Box<dyn Error>> {
        let ch = initilize_mock_repo();
        let update = ch
            .update("user1".to_string(), "ramiro".to_string())
            .unwrap();
        assert!(update);
        Ok(())
    }

    #[test]
    fn update_a_non_existant_key() -> Result<(), Box<dyn Error>> {
        let ch = initilize_mock_repo();
        let update = ch
            .update("user3".to_string(), "ramiro".to_string())
            .unwrap();
        assert!(update);
        Ok(())
    }

    #[test]
    fn find_all_values() -> Result<(), Box<dyn Error>> {
        let ch = initilize_mock_repo();
        let update = ch.find_all().unwrap();

        let mut expected = Vec::new();
        expected.insert(0, "juan".to_string());
        expected.insert(1, "marcos".to_string());
        assert!(expected.iter().all(|item| update.contains(item)));
        Ok(())
    }

    #[test]
    fn see_coordination_between_threads() -> Result<(), Box<dyn Error>> {
        let ch = initilize_mock_repo();
        let ch1 = ch.clone();
        let ch2 = ch.clone();
        thread::spawn(move || {
            let result = ch1.search("user1".to_string()).unwrap().unwrap();
            let expected = "juan".to_string();
            assert_eq!(result, expected);
        });
        thread::spawn(move || {
            let result = ch2.search("user2".to_string()).unwrap().unwrap();
            let expected = "marcos".to_string();
            assert_eq!(result, expected);
        });
        Ok(())
    }
}

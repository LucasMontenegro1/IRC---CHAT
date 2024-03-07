use crate::{
    error::error_server::ErrorServer,
    repository::{
        dao::hash_map_dao::HashMapDao, repo::Repository,
        repository_channel::client_channel::ClientChannel,
    },
};

use std::{
    hash::Hash,
    sync::mpsc,
    thread::{self, JoinHandle},
};

type DefaultResult = Result<(), ErrorServer>;

///
/// Struct that contains the  [`thread::JoinHandle`] in which an initialized
/// [`Repository`] runs Also owns the [`ClientChannel`] which allows the communication
/// between the client and the running repository.
pub struct RepositoryHandler<K, T> {
    pub thread: JoinHandle<DefaultResult>,
    client_channel: ClientChannel<K, T>,
}

impl<K, T> RepositoryHandler<K, T>
where
    K: Eq + PartialEq + Hash + Send + Clone + 'static,
    T: Clone + Send + 'static,
{
    //type RunChannel = ClientChannel<K,T>;
    /// Constructor
    ///
    /// # Returns
    /// If the user succesfully runs the Repository entity, it returns a
    /// entity with the thread in which the repository runs and the channel
    /// that allows externs entities to communicate with the repository initialized,
    /// in order to persist new information.
    pub fn new() -> Result<Self, ErrorServer> {
        let (thread, client_channel) = Self::run()?;
        Ok(RepositoryHandler {
            thread,
            client_channel,
        })
    }

    /// Getter of client_channel field.
    ///
    /// # Returns
    /// Returns a client_channel's clone, which can be used to request/send information
    /// into the Repository handled by the RepositoryHandler.
    pub fn get_channels(&self) -> ClientChannel<K, T> {
        self.client_channel.clone()
    }

    //
    // function that runs the repository
    //
    fn run() -> Result<(JoinHandle<DefaultResult>, ClientChannel<K, T>), ErrorServer> {
        //Canales para comunicarse desde del hilo.
        let (tx, rx) = mpsc::channel::<ClientChannel<K, T>>();

        let thread = thread::spawn(move || -> Result<(), ErrorServer> {
            let repo: Repository<K, T> = Repository::new(Box::new(HashMapDao::new()));
            let channel = repo.get_client_channel();
            //Se envia el canal afuera del hilo.
            tx.send(channel)?;
            repo.run()
        });
        let client = rx.recv()?;
        Ok((thread, client))
    }
}

#[cfg(test)]
mod test {
    use crate::repository::traits::operations::Operations;

    use super::RepositoryHandler;

    #[test]
    fn creates_a_new_database_correctly() {
        let repo: RepositoryHandler<String, String> = RepositoryHandler::new().unwrap();
        let channel = repo.get_channels();
        let result = channel.add("pepe".to_string(), "user1".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn creates_a_new_database_for_generics() {
        let repo: RepositoryHandler<String, bool> = RepositoryHandler::new().unwrap();
        let channel = repo.get_channels();
        let result = channel.add("pepe".to_string(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn default_constructor() {
        let repo = RepositoryHandler::new().unwrap();
        let channel = repo.get_channels();
        let result = channel.add("pepe".to_string(), true);
        assert!(result.is_ok());
    }
}

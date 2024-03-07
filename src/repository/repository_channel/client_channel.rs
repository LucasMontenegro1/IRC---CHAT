use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};

use crate::{
    error::error_server::ErrorServer,
    repository::query::{Query, QueryAnswer},
    repository::traits::operations::Operations,
};

/// Struct that encapsulates the CRUD operations of Repository's clients, in order
/// to abstract entities from manipulating the sending of [`Query`] to the Repository
/// and the reception of the response in the form of [`QueryAnswer`].
#[derive(Debug)]
pub struct ClientChannel<T, Y> {
    tx_client: Sender<Query<T, Y>>,
    rx_client: Arc<Mutex<Receiver<QueryAnswer<Y>>>>,
}

impl<T, Y> Clone for ClientChannel<T, Y> {
    fn clone(&self) -> Self {
        ClientChannel {
            tx_client: self.tx_client.clone(),
            rx_client: Arc::clone(&self.rx_client),
        }
    }
}

impl<T, Y> ClientChannel<T, Y> {
    /// Constructor
    ///
    /// # Arguments
    /// * `tx_client` -  The sending-half of Rust's asynchronous channel type, capable of sending [`Query`] entities.
    /// * `rx_client` -  The receiving-half of Rust's asynchronous channel type, capable of receiving [`QueryAnswer`] entities.
    pub fn new(tx_client: Sender<Query<T, Y>>, rx_client: Receiver<QueryAnswer<Y>>) -> Self {
        ClientChannel {
            tx_client,
            rx_client: Arc::new(Mutex::new(rx_client)),
        }
    }

    fn send_and_receive(&self, query: Query<T, Y>) -> Result<QueryAnswer<Y>, ErrorServer> {
        let lock = match self.rx_client.lock() {
            Ok(c) => c,
            Err(_e) => return Err(ErrorServer::PoisonedThread),
        };

        self.tx_client.send(query)?;

        match lock.recv() {
            Ok(q) => Ok(q),
            Err(_) => Err(ErrorServer::BadQuery),
        }
    }
}

impl<T, Y> Operations<T, Y> for ClientChannel<T, Y> {
    fn add(&self, key: T, value: Y) -> Result<bool, ErrorServer> {
        if let QueryAnswer::Add(r) = self.send_and_receive(Query::add(key, value))? {
            Ok(r)
        } else {
            Err(ErrorServer::BadQuery)
        }
    }
    fn delete(&self, key: T) -> Result<bool, ErrorServer> {
        if let QueryAnswer::Delete(r) = self.send_and_receive(Query::delete(key))? {
            Ok(r)
        } else {
            Err(ErrorServer::BadQuery)
        }
    }
    fn find_all(&self) -> Result<Vec<Y>, ErrorServer> {
        if let QueryAnswer::FindAll(r) = self.send_and_receive(Query::find_all())? {
            Ok(r)
        } else {
            Err(ErrorServer::BadQuery)
        }
    }
    fn search(&self, key: T) -> Result<Option<Y>, ErrorServer> {
        if let QueryAnswer::Search(r) = self.send_and_receive(Query::search(key))? {
            Ok(r)
        } else {
            Err(ErrorServer::BadQuery)
        }
    }
    fn update(&self, key: T, value: Y) -> Result<bool, ErrorServer> {
        if let QueryAnswer::Update(r) = self.send_and_receive(Query::update(key, value))? {
            Ok(r)
        } else {
            Err(ErrorServer::BadQuery)
        }
    }
}

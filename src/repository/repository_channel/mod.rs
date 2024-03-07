use std::sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError};

use crate::repository::{
    query::{Query, QueryAnswer},
    repository_channel::client_channel::ClientChannel,
};
pub mod client_channel;
///
/// Multi-producer multi-consumer channels for message
/// passing.
///
/// Wraps two communication channels, one for sending and receiving the [`Query`]` entity and the other for
/// sending and receiving the [`QueryAnswer`] entity. This channels are used to handle queries to a database
/// and to deliver their responses.
///
pub struct RepoChannel<T, Y> {
    //client_channel: ClientChannel<T, Y>,
    rx_server: Receiver<Query<T, Y>>,
    tx_server: Sender<QueryAnswer<Y>>,
}

//impl<T, Y> Default for RepoChannel<T, Y> {
//fn default() -> Self {
//Self::new()
//}
//}

impl<T, Y> RepoChannel<T, Y> {
    ///
    /// Constructor
    ///
    /// # Returns
    /// Generates two channels of communication, one that enables the sending of [`Query`] entities and the other
    // one send and Returns a RepoChannel
    ///
    pub fn new() -> (RepoChannel<T, Y>, ClientChannel<T, Y>) {
        let (tx_client, rx_server) = mpsc::channel::<Query<T, Y>>();
        let (tx_server, rx) = mpsc::channel::<QueryAnswer<Y>>();

        let client_channel = ClientChannel::new(tx_client, rx);

        (
            RepoChannel {
                tx_server,
                rx_server,
            },
            client_channel.clone(),
        )
    }
    ///
    /// Sends a value through the [`QueryAnswer`] channel.
    ///
    pub fn send(&self, a: QueryAnswer<Y>) -> Result<(), SendError<QueryAnswer<Y>>> {
        self.tx_server.send(a)
    }

    ///
    /// Tries to recieve a value through the [`Query`] channel
    ///
    pub fn try_recv(&self) -> Result<Query<T, Y>, TryRecvError> {
        self.rx_server.try_recv()
    }

    //
    // Returns a cloned value of a [`ClientChannel`] entity contained in the instance.
    // This entity is used to handle the sending of queries and the receiving of a query's answer.
    //
    //pub fn clone_client_channel(&self) -> ClientChannel<T, Y> {
    //self.client_channel.clone()
    //}
}

#[cfg(test)]
mod test {
    use super::{client_channel::ClientChannel, RepoChannel};
    use crate::repository::query::QueryAnswer;

    use std::sync::mpsc::SendError;

    #[test]
    fn can_send_query_answers() -> Result<(), SendError<QueryAnswer<String>>> {
        let (channel, _client_channel): (
            RepoChannel<String, String>,
            ClientChannel<String, String>,
        ) = RepoChannel::new();
        channel.send(QueryAnswer::Search(None))?;
        channel.send(QueryAnswer::Add(true))?;
        channel.send(QueryAnswer::Update(true))?;
        channel.send(QueryAnswer::Delete(true))?;
        channel.send(QueryAnswer::FindAll(vec![]))
    }

    /*
    #[test]
    fn can_send_data_to_repo_channel() -> Result<(), ErrorServer> {
        let channel: RepoChannel<String, String> = RepoChannel::new();
        let client = channel.clone_client_channel();
        client.search("key".to_string())?;
        client.add("key".to_string(),"value".to_string())?;
        client.update("key".to_string(),"value".to_string())?;
        client.delete("key".to_string())?;
        client.find_all()?;
        Ok(())
    }

    #[test]
    fn can_receive_data_on_repo_channel() -> Result<(), Box<dyn Error>> {
        let channel: RepoChannel<String, String> = RepoChannel::new();
        let client = channel.clone_client_channel();
        client.search("key".to_string()).unwrap();
        assert_eq!(Query::search("hola".to_string()), channel.try_recv()?);
        Ok(())
    }
    */
}

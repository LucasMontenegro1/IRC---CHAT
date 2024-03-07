use core::fmt;

use crate::error::error_server::ErrorServer;

use self::edge::Edge;
use self::node::Node;

use super::server::Server;
use super::server_connection::ConnectionServer;

pub mod edge;
pub mod node;

///
/// implementation of
/// a spanning tree
///
pub struct SpanningTree {
    edges: Vec<Edge>,
    root: Node,
}

impl SpanningTree {
    ///
    /// function that creates a spanning
    /// tree, from a set of edges and nodes
    ///
    pub fn new(root: Node, edges: Vec<Edge>) -> Self {
        SpanningTree { root, edges }
    }

    ///
    /// returns the number of nodes
    /// in the spanning tree
    ///
    pub fn get_number_of_nodes(&self) -> usize {
        let unique = &self.get_servers();
        unique.len()
    }

    ///
    /// returns a vector containing
    /// all servers in the tree
    ///
    pub fn get_servers(&self) -> Vec<Server> {
        let mut unique = vec![];
        for edge in &self.edges {
            if !unique.contains(&edge.source.get_element().clone()) {
                unique.push(edge.source.get_element().clone());
            }
            if !unique.contains(&edge.destination.get_element().clone()) {
                unique.push(edge.destination.get_element().clone());
            }
        }
        unique
    }

    ///
    /// returns true if the spanning tree contains the
    /// given servername
    ///
    pub fn contains(&self, servername: String) -> bool {
        let server = Server::new(servername, None);
        self.get_servers().contains(&server)
    }

    pub fn search(&self, name: String) -> Option<Server> {
        let mut unique = vec![];
        let mock = Server::new(name, None);
        for edge in &self.edges {
            if !unique.contains(&edge.source.clone()) {
                unique.push(edge.source.clone());
            }
            if !unique.contains(&edge.destination.clone()) {
                unique.push(edge.destination.clone());
            }
        }
        for node in unique {
            if node.server.clone() == mock {
                return Some(node.server);
            }
        }
        None
    }

    pub fn look_for_nearest_connection(&self, servername: String) -> Option<Server> {
        let server = Server::new(servername, None);
        let node = edge::search_node(
            self.edges.clone(),
            self.get_number_of_nodes(),
            self.root.clone(),
            server,
        );
        if let Some(result) = node {
            return Some(result.server);
        }
        None
    }
    ///
    /// adds a new edge to the spanning tree
    ///
    pub fn add_edge(&mut self, new: Edge) {
        self.edges.push(new);
    }

    ///
    /// deletes an existing edge
    ///
    pub fn delete_edge(&mut self, source: String, destination: String) {
        let new_edges = self.edges.clone();
        self.edges = vec![];
        for edge in new_edges {
            if edge.source.server.servername != source
                || edge.destination.server.servername != destination
            {
                self.edges.push(edge.clone());
            }
        }
    }

    //pub netsplit() {}

    ///
    /// returns all the edges in the tree
    ///
    pub fn get_edges(&self) -> Vec<Edge> {
        self.edges.clone()
    }

    ///
    /// returns the root of the tree
    ///
    pub fn get_root(&self) -> Node {
        self.root.clone()
    }

    ///
    /// returns the root's name
    ///
    pub fn get_root_name(&self) -> String {
        self.root.get_element().servername
    }

    ///
    /// set connection on a node
    ///
    pub fn set_connection(
        &mut self,
        name: String,
        connection: ConnectionServer,
    ) -> Result<(), ErrorServer> {
        let node = self.look_for_nearest_connection(name.clone());
        let server = Server::new(name.clone(), Some(connection));
        let edge = Edge::new(self.get_root(), Node::new(server), 1);
        match node {
            None => {}
            Some(s) => {
                if s.servername == name {
                    self.delete_edge(self.get_root().server.servername, name);
                }
            }
        }
        self.add_edge(edge);
        Ok(())
    }
}

impl fmt::Debug for SpanningTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "DEBUGGING SPANNING TREE")?;
        writeln!(f, "(Origen->Destino) | Peso")?;
        for e in self.get_edges() {
            writeln!(
                f,
                "({:?}->{:?}) | {:?}",
                e.source.server.servername, e.destination.server.servername, e.cost
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::server_comunication::server::Server;

    use super::{edge::Edge, node::Node, SpanningTree};

    #[test]
    fn number_of_nodes() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let edge3 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge2 = Edge::new(node3.clone(), node2.clone(), 2);
        let edge1 = Edge::new(node1.clone(), node3.clone(), 10000);
        let edges = vec![edge1.clone(), edge2.clone(), edge3.clone()];

        let st = SpanningTree::new(node1.clone(), edges);
        assert_eq!(3, st.get_number_of_nodes());
    }

    #[test]
    fn add_edge() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let edge3 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge2 = Edge::new(node3.clone(), node2.clone(), 2);
        let edge1 = Edge::new(node1.clone(), node3.clone(), 10000);
        let edges = vec![edge1.clone(), edge2.clone(), edge3.clone()];
        let node4 = Node::new(Server::new("server4".to_string(), None));
        let edge4 = Edge::new(node3.clone(), node4.clone(), 1);

        let mut st = SpanningTree::new(node1.clone(), edges);
        st.add_edge(edge4);
        assert_eq!(4, st.get_number_of_nodes());
    }

    #[test]
    fn get_servers() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let edge3 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge2 = Edge::new(node3.clone(), node2.clone(), 2);
        let edge1 = Edge::new(node1.clone(), node3.clone(), 10000);
        let edges = vec![edge1.clone(), edge2.clone(), edge3.clone()];
        let st = SpanningTree::new(node1, edges);
        let result = st.get_servers();
        let expected = vec![
            Server::new("server1".to_string(), None),
            Server::new("server3".to_string(), None),
            Server::new("server2".to_string(), None),
        ];

        assert_eq!(result, expected)
    }

    #[test]
    fn contains_server() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let edge3 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge2 = Edge::new(node3.clone(), node2.clone(), 2);
        let edge1 = Edge::new(node1.clone(), node3.clone(), 1);
        let edges = vec![edge1.clone(), edge2.clone(), edge3.clone()];
        let st = SpanningTree::new(node1, edges);

        let contained = "server3".to_string();
        assert!(st.contains(contained));
    }

    #[test]
    fn look_for_nearest_connection() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let edge1 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge2 = Edge::new(node2.clone(), node3.clone(), 1);
        let edges = vec![edge1.clone(), edge2.clone()];
        let st = SpanningTree::new(node1, edges);
        let result = st
            .look_for_nearest_connection("server3".to_string())
            .unwrap();
        assert_eq!(result, Server::new("server2".to_string(), None));
    }

    #[test]
    fn look_for_nearest_connection_2() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let node4 = Node::new(Server::new("server4".to_string(), None));
        let edge1 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge2 = Edge::new(node2.clone(), node3.clone(), 1);
        let edge3 = Edge::new(node2.clone(), node4.clone(), 1);
        let edges = vec![edge1.clone(), edge2.clone(), edge3.clone()];
        let st = SpanningTree::new(node1, edges);
        let result = st
            .look_for_nearest_connection("server4".to_string())
            .unwrap();
        assert_eq!(result, Server::new("server2".to_string(), None));
    }

    #[test]
    fn look_for_nearest_connection_3() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let edge1 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge2 = Edge::new(node1.clone(), node3.clone(), 1);
        let edges = vec![edge1.clone(), edge2.clone()];
        let st = SpanningTree::new(node1, edges);
        let result = st
            .look_for_nearest_connection("server3".to_string())
            .unwrap();
        assert_eq!(result, Server::new("server3".to_string(), None));
    }
    #[test]

    fn look_for_nearest_connection_4() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let edge1 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge2 = Edge::new(node1.clone(), node3.clone(), 1);
        let edges = vec![edge1.clone(), edge2.clone()];
        let st = SpanningTree::new(node1, edges);
        let result = st
            .look_for_nearest_connection("server1".to_string())
            .unwrap();
        assert_eq!(result, Server::new("server1".to_string(), None));
    }
}

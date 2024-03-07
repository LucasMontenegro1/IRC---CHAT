use crate::server_comunication::server::Server;

use super::node::Node;

///
/// implements an edge that
/// is used for communication
/// between different servers within
/// the irc network
///
#[derive(Debug, PartialEq, Clone)]
pub struct Edge {
    pub source: Node,
    pub destination: Node,
    pub cost: usize,
}

impl Edge {
    ///
    /// creates a new edge
    /// connecting two nodes
    /// with their respective
    /// weights
    ///
    pub fn new(source: Node, destination: Node, cost: usize) -> Self {
        Self {
            source,
            destination,
            cost,
        }
    }
}

///
/// Kruscal algorithm
/// that given a set of edges (a graph)
/// finds the minimum tree.
///
pub fn kruskal(mut edges: Vec<Edge>, number_of_nodes: usize) -> Option<Vec<Edge>> {
    let mut visited_nodes: Vec<Node> = Vec::new();
    let mut tree: Vec<Edge> = Vec::new();
    edges.sort_by(|a, b| a.cost.cmp(&b.cost));
    for edge in edges {
        if !visited_nodes.contains(&edge.source) || !visited_nodes.contains(&edge.destination) {
            visited_nodes.push(edge.source.clone());
            visited_nodes.push(edge.destination.clone());
            tree.push(edge);
        }
        if number_of_nodes - 1 == tree.len() {
            return Some(tree);
        }
    }
    None
}

///
/// given a graph, calculates the distance
/// between two nodes belonging to the graph
/// if there is a tree returns some
///
pub fn distance_between_servers(
    edges: Vec<Edge>,
    number_of_nodes: usize,
    initial: Node,
    searched: Server,
) -> Option<usize> {
    let tree = kruskal(edges, number_of_nodes);
    tree.map(|c| minimum_distance(c, initial, searched))
}

///
/// returns the distance between two nodes
/// in case there isn't a path between
/// them it returns distance 0
///
pub fn minimum_distance(tree: Vec<Edge>, initial: Node, searched: Server) -> usize {
    let mut distance = 0;
    let mut c = vec![];
    let tree_copy = tree.clone();
    for edge in tree {
        if edge.source == initial {
            c.push(edge.clone());
        }
    }

    if c.is_empty() {
        return distance;
    }
    let cloned_searched = searched.clone();
    let cloned_searched2 = searched;

    for edge in c {
        if edge.destination.server == cloned_searched {
            distance += edge.cost;
            return distance;
        } else {
            let aux = minimum_distance(
                tree_copy.clone(),
                edge.destination.clone(),
                cloned_searched2.clone(),
            );
            if aux == 0 {
                distance = 0;
            } else {
                distance += aux + edge.cost;
                return distance;
            }
        }
    }
    distance
}

///
/// function that returns the closest
/// node to the main server that leads
/// to the destination node
///
///
pub fn search_node(
    edges: Vec<Edge>,
    number_of_nodes: usize,
    initial: Node,
    searched: Server,
) -> Option<Node> {
    let tree = kruskal(edges, number_of_nodes);
    match tree {
        Some(c) => look_for_nearest_connection(c, initial, searched),
        None => None,
    }
}

///
/// searches for the nearest connection
/// between two servers
///
pub fn look_for_nearest_connection(
    tree: Vec<Edge>,
    initial: Node,
    searched: Server,
) -> Option<Node> {
    if initial.server == searched {
        return Some(initial);
    }
    let mut c = vec![];
    let tree_copy = tree.clone();
    for edge in tree {
        if edge.source == initial {
            c.push(edge.clone());
        }
    }

    if c.is_empty() {
        return None;
    }
    let cloned_searched = searched.clone();
    let cloned_searched2 = searched;

    for edge in c {
        if edge.destination.server == cloned_searched {
            return Some(edge.destination);
        } else {
            let d = look_for_nearest_connection(
                tree_copy.clone(),
                edge.destination.clone(),
                cloned_searched2.clone(),
            );
            if d.is_some() {
                return Some(edge.destination);
            }
        }
    }

    None
}

#[cfg(test)]

mod test {
    use crate::server_comunication::server::Server;

    use super::*;
    #[test]
    fn does_kruskal_correctly_with_three_nodes() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let edge3 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge2 = Edge::new(node3.clone(), node2.clone(), 2);
        let edge1 = Edge::new(node1.clone(), node3.clone(), 10000);
        let edges = vec![edge1.clone(), edge2.clone(), edge3.clone()];
        vec![node1, node2, node3];
        let result = kruskal(edges, 3);
        assert_eq!(result, Some(vec![edge3.clone(), edge2.clone()]))
    }

    #[test]
    fn does_kruskal_correctly() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let node4 = Node::new(Server::new("server4".to_string(), None));
        let edge6 = Edge::new(node4.clone(), node1.clone(), 1);
        let edge5 = Edge::new(node4.clone(), node2.clone(), 2);
        let edge4 = Edge::new(node4.clone(), node3.clone(), 3);
        let edge3 = Edge::new(node1.clone(), node2.clone(), 4);
        let edge2 = Edge::new(node3.clone(), node2.clone(), 5);
        let edge1 = Edge::new(node1.clone(), node3.clone(), 10000);
        let edges = vec![
            edge1.clone(),
            edge2.clone(),
            edge3.clone(),
            edge4.clone(),
            edge5.clone(),
            edge6.clone(),
        ];
        vec![node1, node2, node3, node4];
        let result = kruskal(edges, 4);
        assert_eq!(
            result,
            Some(vec![edge6.clone(), edge5.clone(), edge4.clone()])
        )
    }

    #[test]
    fn searches_correctly() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let node4 = Node::new(Server::new("server4".to_string(), None));
        let node5 = Node::new(Server::new("server5".to_string(), None));
        let edge1 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge2 = Edge::new(node1.clone(), node3.clone(), 1);
        let edge3 = Edge::new(node2.clone(), node4.clone(), 1);
        let edge4 = Edge::new(node4.clone(), node5.clone(), 1);
        let edges = vec![edge1.clone(), edge2.clone(), edge3.clone(), edge4.clone()];
        let searched = Server::new("server5".to_string(), None);
        let x = search_node(edges, 5, node1, searched);
        let node = Node::new(Server::new("server2".to_string(), None));
        match x {
            Some(c) => {
                assert_eq!(node, c);
            }
            None => panic!(),
        }
    }

    #[test]
    fn search_correctly_ad() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let node4 = Node::new(Server::new("server4".to_string(), None));
        let node5 = Node::new(Server::new("server5".to_string(), None));
        let edge1 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge2 = Edge::new(node1.clone(), node3.clone(), 1);
        let edge3 = Edge::new(node2.clone(), node4.clone(), 1);
        let edge4 = Edge::new(node4.clone(), node5.clone(), 1);
        let edges = vec![edge1.clone(), edge2.clone(), edge3.clone(), edge4.clone()];
        let searched = Server::new("server3".to_string(), None);
        let x = search_node(edges, 5, node1, searched);
        let node = Node::new(Server::new("server3".to_string(), None));
        match x {
            Some(c) => {
                assert_eq!(node, c);
            }
            None => panic!(),
        }
    }

    #[test]
    fn distance() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node4 = Node::new(Server::new("server4".to_string(), None));
        let node5 = Node::new(Server::new("server5".to_string(), None));
        let edge1 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge3 = Edge::new(node2.clone(), node4.clone(), 1);
        let edge4 = Edge::new(node4.clone(), node5.clone(), 1);
        let edges = vec![edge1.clone(), edge3.clone(), edge4.clone()];
        let searched = Server::new("server5".to_string(), None);
        let result = distance_between_servers(edges, 4, node1, searched).unwrap();
        assert_eq!(result, 3);
    }

    #[test]
    fn distance_complex() {
        let node1 = Node::new(Server::new("server1".to_string(), None));
        let node2 = Node::new(Server::new("server2".to_string(), None));
        let node4 = Node::new(Server::new("server4".to_string(), None));
        let node3 = Node::new(Server::new("server3".to_string(), None));
        let node5 = Node::new(Server::new("server5".to_string(), None));
        let node6 = Node::new(Server::new("server6".to_string(), None));
        let edge1 = Edge::new(node1.clone(), node2.clone(), 1);
        let edge3 = Edge::new(node1.clone(), node3.clone(), 1);
        let edge4 = Edge::new(node3.clone(), node4.clone(), 1);
        let edge5 = Edge::new(node3.clone(), node5.clone(), 1);
        let edge6 = Edge::new(node2.clone(), node6.clone(), 1);
        let edges = vec![
            edge1.clone(),
            edge3.clone(),
            edge4.clone(),
            edge5.clone(),
            edge6.clone(),
        ];
        let searched = Server::new("server5".to_string(), None);
        let result = distance_between_servers(edges, 6, node1, searched).unwrap();
        assert_eq!(result, 2);
    }
}

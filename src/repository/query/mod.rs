use crate::error::error_server::ErrorServer;

/// Enum that are used to classify the [`Query`] entity. Each one represents a CRUD operation.
#[derive(Debug, PartialEq, Eq)]
pub enum QueryOption {
    Search,
    Delete,
    Update,
    Add,
    FindAll,
}
/// Enum representing the type of response for each CRUD operation made through a [`Query`] entity.
/// Each variant contains different types of entities with which it responds.
#[derive(Debug, PartialEq, Eq)]
pub enum QueryAnswer<T> {
    Search(Option<T>),
    Delete(bool),
    Update(bool),
    Add(bool),
    FindAll(Vec<T>),
}

///
/// Struct that represents a query to a persistance entity, such as a database.
/// In our program you can see the [`crate::repository::repo::Repository`] entity, whose implementation
/// is compatible with this entity.
///
#[derive(Debug, PartialEq, Eq)]
pub struct Query<T, R> {
    option: QueryOption,
    argument1: Option<T>,
    argument2: Option<R>,
}

impl<T, R> Query<T, R> {
    ///
    /// Function that creates a new query
    ///
    /// # Arguments
    /// * `option:QueryOption` : the desired query
    /// * `argument1: Option<T>` : element of the desired query
    /// * `argument2: Option<R>` : second element (see kind of querys in QueryOption and rest of functions)
    ///
    ///
    ///  # Example
    ///  ```rust
    ///   use irc_project::repository::query::Query;
    ///   use irc_project::repository::query::QueryOption;
    ///   let q: Query<String, String> = Query::new(QueryOption::Search, Some("key".to_string()), None);
    ///     
    ///  ```
    ///
    pub fn new(option: QueryOption, argument1: Option<T>, argument2: Option<R>) -> Self {
        Query {
            option,
            argument1,
            argument2,
        }
    }

    ///
    /// returns the option of the query
    ///
    pub fn get_option(&self) -> &QueryOption {
        &self.option
    }

    ///
    /// returns the arguments of the query as a tuple
    ///
    pub fn get_arguments(self) -> (Option<T>, Option<R>) {
        (self.argument1, self.argument2)
    }

    ///
    /// returns a new query of the type search
    ///
    pub fn search(argument1: T) -> Self {
        Query::new(QueryOption::Search, Some(argument1), None)
    }

    ///
    /// returns a new query of the type delete
    ///
    pub fn delete(argument1: T) -> Self {
        Query::new(QueryOption::Delete, Some(argument1), None)
    }

    ///
    /// returns a new query of the type update
    ///
    pub fn update(argument1: T, argument2: R) -> Self {
        Query::new(QueryOption::Update, Some(argument1), Some(argument2))
    }

    ///
    /// returns a new query of the type add
    ///
    pub fn add(argument1: T, argument2: R) -> Self {
        Query::new(QueryOption::Add, Some(argument1), Some(argument2))
    }

    ///
    /// returns a new query of the type find_all
    ///
    pub fn find_all() -> Self {
        Query::new(QueryOption::FindAll, None, None)
    }

    ///
    /// Fuction that validates that a response of the database it's coherent and handle.
    /// it as a [`Result`].
    ///
    /// # Arguments
    /// * `response` : database's response, represented by a [`QueryAnswer`]
    ///
    /// # Returns
    ///   If the response corresponds to the correct [`QueryOption`] contained
    /// in the query, returns the response wrapped in a Result variant. If not, handles
    /// as an [`ErrorServer`].
    ///
    ///
    pub fn validate_response(
        &self,
        response: QueryAnswer<R>,
    ) -> Result<QueryAnswer<R>, ErrorServer> {
        let result = match &response {
            QueryAnswer::Search(_) => self.option == QueryOption::Search,
            QueryAnswer::Add(_) => self.option == QueryOption::Add,
            QueryAnswer::Delete(_) => self.option == QueryOption::Delete,
            QueryAnswer::FindAll(_) => self.option == QueryOption::FindAll,
            QueryAnswer::Update(_) => self.option == QueryOption::Update,
        };

        if result {
            Ok(response)
        } else {
            Err(ErrorServer::BadQuery)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::repository::query::QueryAnswer;

    use super::{Query, QueryOption};

    #[test]
    fn query_options_are_created_properly() {
        let _oper = QueryOption::Search;
        assert!(matches!(QueryOption::Search, _oper))
    }

    #[test]
    fn query_search() {
        let result: Query<String, String> = Query::search("key".to_string());
        let expected = Query {
            option: QueryOption::Search,
            argument1: Some(String::from("key")),
            argument2: None,
        };
        assert_eq!(result, expected)
    }

    #[test]
    fn query_delete() {
        let result: Query<String, String> = Query::delete("key".to_string());
        let expected = Query {
            option: QueryOption::Delete,
            argument1: Some(String::from("key")),
            argument2: None,
        };
        assert_eq!(result, expected)
    }

    #[test]
    fn query_update() {
        let result: Query<String, String> = Query::update("key".to_string(), "value".to_string());
        let expected = Query {
            option: QueryOption::Update,
            argument1: Some(String::from("key")),
            argument2: Some(String::from("value")),
        };
        assert_eq!(result, expected)
    }
    #[test]
    fn query_add() {
        let result: Query<String, String> = Query::add("key".to_string(), "value".to_string());
        let expected = Query {
            option: QueryOption::Add,
            argument1: Some(String::from("key")),
            argument2: Some(String::from("value")),
        };
        assert_eq!(result, expected)
    }

    #[test]
    fn query_find_all() {
        let result: Query<String, String> = Query::find_all();
        let expected = Query {
            option: QueryOption::FindAll,
            argument1: None,
            argument2: None,
        };
        assert_eq!(result, expected)
    }

    #[test]
    fn get_an_option_from_query() {
        let q: Query<String, String> =
            Query::new(QueryOption::Search, Some("key".to_string()), None);
        let _oper = q.get_option();

        assert!(matches!(&QueryOption::Search, _oper));
    }

    #[test]
    fn debugs_correctly() {
        let origin: Query<String, String> = Query::find_all();
        let origin2 = QueryOption::FindAll;
        let origin3: QueryAnswer<String> = QueryAnswer::FindAll(vec![]);
        assert_eq!(
            format!("{origin:?}"),
            "Query { option: FindAll, argument1: None, argument2: None }"
        );
        assert_eq!(format!("{origin2:?}"), "FindAll");
        assert_eq!(format!("{origin3:?}"), "FindAll([])");
    }
}

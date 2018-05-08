use futures::prelude::*;
use http::status::StatusCode;
use hyper::{Body, Response};

use thegraph::common::query::QueryResult;
use thegraph::common::server::GraphQLServerError;

/// Future for HTTP responses to GraphQL query requests.
pub struct GraphQLResponse {
    result: Result<QueryResult, GraphQLServerError>,
}

impl GraphQLResponse {
    /// Creates a new GraphQLResponse future based on the result generated by
    /// running a query.
    pub fn new(result: Result<QueryResult, GraphQLServerError>) -> Self {
        GraphQLResponse { result }
    }
}

impl Future for GraphQLResponse {
    type Item = Response<Body>;
    type Error = GraphQLServerError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let response = match self.result {
            Ok(ref result) => {
                let data = format!("{:?}", result);
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(data))
                    .unwrap()
            }
            Err(ref e) => {
                let data = format!("{}", e);
                Response::builder()
                    .status(match e {
                        &GraphQLServerError::ClientError(_) => StatusCode::BAD_REQUEST,
                        &GraphQLServerError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                        &GraphQLServerError::QueryError(_) => StatusCode::BAD_REQUEST,
                        &GraphQLServerError::Canceled(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    })
                    .body(Body::from(data))
                    .unwrap()
            }
        };

        Ok(Async::Ready(response))
    }
}

#[cfg(test)]
mod tests {
    use graphql_parser;
    use http::status::StatusCode;
    use tokio_core::reactor::Core;

    use super::GraphQLResponse;
    use thegraph::common::query::{QueryError, QueryResult};
    use thegraph::common::server::GraphQLServerError;

    #[test]
    fn generates_500_for_internal_errors() {
        let mut core = Core::new().unwrap();
        let request = GraphQLResponse::new(Err(GraphQLServerError::from("Some error")));
        let result = core.run(request);
        let response = result.expect("Should generate a response");
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn generates_401_for_client_errors() {
        let mut core = Core::new().unwrap();
        let request =
            GraphQLResponse::new(Err(GraphQLServerError::ClientError(String::from("foo"))));
        let result = core.run(request);
        let response = result.expect("Should generate a response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn generates_401_for_query_errors() {
        let mut core = Core::new().unwrap();
        let parse_error = graphql_parser::parse_query("<>?><").unwrap_err();
        let query_error = QueryError::from(parse_error);
        let request = GraphQLResponse::new(Err(GraphQLServerError::from(query_error)));
        let result = core.run(request);
        let response = result.expect("Should generate a response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn generates_200_for_query_results() {
        let mut core = Core::new().unwrap();
        let query_result = QueryResult {};
        let request = GraphQLResponse::new(Ok(query_result));
        let result = core.run(request);
        let response = result.expect("Should generate a response");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
use rocket::State;
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::json::Json;

use aw_models::Query;

use aw_query;
use aw_query::QueryError;
use crate::endpoints::ServerState;

#[derive(Serialize)]
struct QueryErrorJson {
    status: u16,
    reason: String,
    message: String
}

/* TODO: Slightly ugly code with ok() and error() */

fn ok(data: Vec<aw_query::DataType>) -> status::Custom<Json<Vec<aw_query::DataType>>> {
    status::Custom(Status::Ok, Json(data))
}

enum PossibleQueryErrors {
    QueryErrorJson,
    QueryError
}

// TODO: Add openAPI support and the [openapi] macro
#[post("/", data = "<query_req>")]
pub fn query(query_req: Json<Query>, state: State<ServerState>) -> Result<Json<Vec<aw_query::DataType>>, Status> {
    let query_code = query_req.0.query.join("\n");
    let intervals = &query_req.0.timeperiods;
    let mut results = Vec::new();
    for interval in intervals {
        // Cannot re-use endpoints_get_lock!() here because it returns Err(Status) on failure and this
        // function returns status::Custom
        let datastore = match state.datastore.lock() {
            Ok(ds) => ds,
            Err(e) => {
                warn!("Taking datastore lock failed, returning 500: {}", e);
                return Err(Status::ServiceUnavailable);
            }
        };
        let result = match aw_query::query(&query_code, &interval, &datastore) {
            Ok(data) => data,
            Err(e) => {
                warn!("Query failed: {:?}", e);
                return Err(Status::NoContent);
            }
        };
        results.push(result);
    }
    Ok(Json(results))
}

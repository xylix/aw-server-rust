extern crate aw_server;

extern crate chrono;
extern crate serde_json;

#[cfg(test)]
mod query_tests {
    use chrono;
    use chrono::Duration;
    use serde_json::json;

    use aw_server::query;
    use aw_server::query::QueryError;
    use aw_server::query::DataType;
    use aw_server::datastore::Datastore;
    use aw_server::models::Bucket;
    use aw_server::models::Event;
    use aw_server::models::TimeInterval;

    static TIME_INTERVAL : &str = "2000-01-01T00:00:00Z/2000-01-02T00:00:00Z";

    fn setup_datastore_empty() -> Datastore {
        Datastore::new_in_memory()
    }

    fn setup_datastore_populated() -> Datastore {
        let ds = Datastore::new_in_memory();
        // Create bucket
        let bucket = Bucket {
            bid: None,
            id: "testid".to_string(),
            _type: "testtype".to_string(),
            client: "testclient".to_string(),
            hostname: "testhost".to_string(),
            created: Some(chrono::Utc::now()),
        };
        ds.create_bucket(&bucket).unwrap();
        // Insert events
        let e1 = Event {
            id: None,
            timestamp: chrono::Utc::now(),
            duration: Duration::seconds(0),
            data: json!({"key": "value"})
        };
        let mut e2 = e1.clone();
        e2.timestamp = chrono::Utc::now();
        let mut e_replace = e2.clone();
        e_replace.data = json!({"key": "value2"});
        e_replace.duration = Duration::seconds(2);

        let mut event_list = Vec::new();
        event_list.push(e1.clone());
        event_list.push(e2.clone());

        ds.insert_events(&bucket.id, &event_list).unwrap();

        return ds;
    }

    #[test]
    fn test_number() {
        let ds = setup_datastore_empty();
        let interval = TimeInterval::new_from_string(TIME_INTERVAL).unwrap();

        let code = String::from("1;1.;1.1;");
        query::query(&code, &interval, &ds).unwrap();
    }

    #[test]
    fn test_return() {
        let ds = setup_datastore_empty();
        let interval = TimeInterval::new_from_string(TIME_INTERVAL).unwrap();

        let code = String::from("return 1;");
        query::query(&code, &interval, &ds).unwrap();

        let code = String::from("return 1+1;");
        query::query(&code, &interval, &ds).unwrap();
    }

    #[test]
    fn test_function() {
        let ds = setup_datastore_empty();
        let interval = TimeInterval::new_from_string(TIME_INTERVAL).unwrap();

        let code = String::from("print(1);");
        query::query(&code, &interval, &ds).unwrap();

        let code = String::from("no_such_function(1);");
        match query::query(&code, &interval, &ds) {
            Ok(ok) => panic!(format!("Expected QueryError, got {:?}", ok)),
            Err(e) => match e {
                QueryError::VariableNotDefined(qe) => assert_eq!(qe, "no_such_function"),
                qe => panic!(format!("Expected QueryError::VariableNotDefined, got {:?}", qe))
            }
        }

        let code = String::from("invalid_type=1; invalid_type(1);");
        match query::query(&code, &interval, &ds) {
            Ok(ok) => panic!(format!("Expected QueryError, got {:?}", ok)),
            Err(e) => match e {
                QueryError::InvalidType(qe) => assert_eq!(qe, "invalid_type"),
                qe => panic!(format!("Expected QueryError::VariableNotDefined, got {:?}", qe))
            }
        }
    }

    #[test]
    fn test_all_functions() {
        let ds = setup_datastore_populated();
        let interval = TimeInterval::new_from_string(TIME_INTERVAL).unwrap();

        let code = String::from("query_bucket(\"testid\");");
        query::query(&code, &interval, &ds).unwrap();
    }

    #[test]
    fn test_string() {
        let ds = setup_datastore_empty();
        let interval = TimeInterval::new_from_string(TIME_INTERVAL).unwrap();

        let code = String::from("a=\"test\";");
        let ret = query::query(&code, &interval, &ds).unwrap();
        match ret {
            query::DataType::String(s) => assert_eq!(s, "test"),
            _ => panic!("Wrong datatype")
        }
    }

    #[test]
    fn test_list() {
        let ds = setup_datastore_empty();
        let interval = TimeInterval::new_from_string(TIME_INTERVAL).unwrap();

        let code = String::from("[];");
        query::query(&code, &interval, &ds).unwrap();

        let code = String::from("[1];");
        query::query(&code, &interval, &ds).unwrap();

        let code = String::from("[1+1];");
        query::query(&code, &interval, &ds).unwrap();

        let code = String::from("[1,1];");
        query::query(&code, &interval, &ds).unwrap();

        let code = String::from("[1,1+2];");
        query::query(&code, &interval, &ds).unwrap();

        let code = String::from("[1,1+1,1+2+3,4/3,[1+2]];");
        query::query(&code, &interval, &ds).unwrap();
    }

    #[test]
    fn test_comment() {
        let ds = setup_datastore_empty();
        let interval = TimeInterval::new_from_string(TIME_INTERVAL).unwrap();

        let code = String::from("1;# testing 123");
        query::query(&code, &interval, &ds).unwrap();
    }

    #[test]
    fn test_dict() {
        let ds = setup_datastore_empty();
        let interval = TimeInterval::new_from_string(TIME_INTERVAL).unwrap();

        let code = String::from("return {};");
        query::query(&code, &interval, &ds).unwrap();

        let code = String::from("return {\"test\": 2};");
        query::query(&code, &interval, &ds).unwrap();

        let code = String::from("return {\"test\": 2, \"test2\": \"teststr\"};");
        query::query(&code, &interval, &ds).unwrap();

        let code = String::from("return {\"test\": {\"test\": \"test\"}};");
        query::query(&code, &interval, &ds).unwrap();
    }

    #[test]
    fn test_math() {
        let ds = setup_datastore_empty();
        let interval = TimeInterval::new_from_string(TIME_INTERVAL).unwrap();

        let code = String::from("1+1;");
        match query::query(&code, &interval, &ds) {
            Ok(r_type) => match r_type {
                DataType::Number(n) => assert_eq!(n, 2.0),
                num => panic!("Expected number, got {:?}", num)
            },
            Err(e) => panic!("Expected number, got {:?}", e)
        };

        let code = String::from("1-1;");
        match query::query(&code, &interval, &ds) {
            Ok(r_type) => match r_type {
                DataType::Number(n) => assert_eq!(n, 0.0),
                num => panic!("Expected number, got {:?}", num)
            },
            Err(e) => panic!("Expected number, got {:?}", e)
        };

        let code = String::from("3*5;");
        match query::query(&code, &interval, &ds) {
            Ok(r_type) => match r_type {
                DataType::Number(n) => assert_eq!(n, 15.0),
                num => panic!("Expected number, got {:?}", num)
            },
            Err(e) => panic!("Expected number, got {:?}", e)
        };

        let code = String::from("4/2;");
        match query::query(&code, &interval, &ds) {
            Ok(r_type) => match r_type {
                DataType::Number(n) => assert_eq!(n, 2.0),
                num => panic!("Expected number, got {:?}", num)
            },
            Err(e) => panic!("Expected number, got {:?}", e)
        };

        let code = String::from("1/0;");
        match query::query(&code, &interval, &ds) {
            Err(e) => match e {
                QueryError::MathError(_) => (),
                qe => panic!("Expected division by zero error, got {:?}", qe)
            }
            Ok(res) => panic!("Expected division by zero error, got {:?}", res)
        };

        let code = String::from("2.5%1;");
        match query::query(&code, &interval, &ds) {
            Ok(r_type) => match r_type {
                DataType::Number(n) => assert_eq!(n, 0.5),
                num => panic!("Expected number, got {:?}", num)
            },
            Err(e) => panic!("Expected number, got {:?}", e)
        };
    }
}
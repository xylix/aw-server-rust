use crate::TimeInterval;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema, Clone, Debug)]
pub struct Query {
    //#[serde(with = "DurationSerialization")]
    pub timeperiods: Vec<TimeInterval>,
    pub query: Vec<String>,
}

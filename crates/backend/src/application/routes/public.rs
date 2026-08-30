use pkg::displayable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum PublicRoute {
    #[serde(rename = "/users/challenge")]
    GetChallenge,
    #[serde(rename = "/users/login")]
    SubmitAnswer,
    #[serde(rename = "/health-check")]
    Health,
}
displayable!(PublicRoute);

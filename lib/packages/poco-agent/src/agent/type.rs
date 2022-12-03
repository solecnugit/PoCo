use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Serialize, Deserialize, Debug, PartialEq, Display)]
pub enum RoundStatus {
    Running,
    Pending,
}
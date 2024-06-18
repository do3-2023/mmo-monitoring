use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePersonDto {
    pub last_name: String,
    pub phone_number: String,
    pub location: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: i32,
    pub last_name: String,
    pub phone_number: String,
    #[serde(default)]
    pub location: String,
}

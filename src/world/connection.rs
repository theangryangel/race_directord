use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Connection {
    pub uname: String,
    pub pname: String,
}

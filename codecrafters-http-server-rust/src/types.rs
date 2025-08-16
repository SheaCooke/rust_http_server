pub struct RequestState {
    pub close_connection: bool,
    pub accept_encoding: String
}

impl Default for RequestState {
    fn default() -> Self {
        RequestState {
            close_connection: true,
            accept_encoding: String::new()
        }
    }
}
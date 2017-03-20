pub struct Database {
}

impl Database {
    pub fn new() -> Database {
        Database {}
    }

    pub fn list(&self) -> Vec<String> {
        [ "1", "2", "3" ].iter().map(|&s| s.into()).collect()
    }
}

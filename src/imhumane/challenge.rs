use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone)]
pub struct Challenge {
    pub id: String,
    pub image: Vec<u8>,
    pub topic: String,
    pub answer: u32,
}

impl Display for Challenge {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "ID: {}, topic: {}, answer: {}", self.id, self.topic, self.answer)
    }
}

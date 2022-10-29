#[derive(Clone, Debug, PartialEq)]
pub struct Token<T> {
    pub row: i32,
    pub column: i32,
    pub value: T,
}

impl<T> Token<T> {
    pub fn new(row: i32, column: i32, value: T) -> Self {
        Self { row, column, value }
    }
}

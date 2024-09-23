///
/// Counts errors by calling method 'add()'
/// - returns Ok if 'limit' of errors is not exceeded
/// - returns Err if count of errors >= 'limit'
pub struct ErrorLimit {
    errors: usize,
    limit: usize,
}
//
// 
impl ErrorLimit {
    ///
    /// Creates new instance of the ErrorLimit wir the [limit]
    pub fn new(limit: usize) -> Self {
        Self { errors: limit, limit }
    }
    ///
    /// Counts errors
    /// - returns Ok if 'limit' of errors is not exceeded
    /// - returns Err if count of errors >= 'limit'
    pub fn add(&mut self) -> Result<(), ()> {
        if self.errors > 0 {
            self.errors -= 1;
            Ok(())
        } else {
            Err(())
        }
    }
    ///
    /// Reset counter
    pub fn reset(&mut self) {
        self.errors = self.limit;
    }
    ///
    /// Returns limit (the number of allowed calls add() method)
    pub fn limit(&self) -> usize {
        self.limit
    }
    ///
    /// Returns current number of errors
    pub fn errors(&self) -> usize {
        self.errors
    }
}
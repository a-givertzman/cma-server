///
/// Counts errors by calling method 'add()'
/// - returns Ok if 'limit' of errors is not exceeded
/// - returns Err if count of errors >= 'limit'
pub struct ErrorLimit {
    value: usize,
    limit: usize,
}
//
// 
impl ErrorLimit {
    ///
    /// Creates new instance of the ErrorLimit wir the [limit]
    pub fn new(limit: usize) -> Self {
        Self { value: limit, limit }
    }
    ///
    /// Counts errors
    /// - returns Ok if 'limit' of errors is not exceeded
    /// - returns Err if count of errors >= 'limit'
    pub fn add(&mut self) -> Result<(), ()> {
        if self.value > 0 {
            self.value -= 1;
            Ok(())
        } else {
            Err(())
        }
    }
    ///
    /// Reset counter
    pub fn reset(&mut self) {
        self.value = self.limit;
    }
    ///
    /// Returns limit (the number of allowed calls add() method)
    pub fn limit(&self) -> usize {
        self.limit
    }
}
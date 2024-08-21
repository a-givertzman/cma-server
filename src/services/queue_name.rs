use std::cell::RefCell;

///
/// Contains Name of the Queue in the separate format
/// Service.Queue -> Service & Queue
#[derive(Debug, Clone, PartialEq)]
pub struct QueueName {
    name: String,
    service: RefCell<Option<String>>,
    queue: RefCell<Option<String>>,
}
///
/// Contains the Service's queue name in the format 'Servece.queue'
impl QueueName {
    ///
    /// Creates new instance of the QueueName from the string like "Service.Queue"
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            service: RefCell::new(None),
            queue: RefCell::new(None),
        }
    }
    ///
    /// Returns original name
    pub fn name(&self) -> String {
        self.name.clone()
    }
    ///
    /// Splits name 'Service.queue' into 
    /// - Service
    /// - queue
    fn split_(&self) -> Result<(), String> {
        let parts: Vec<&str> = self.name.split('.').collect();
        match parts.first() {
            Some(value) => {
                self.service.borrow_mut().replace(value.to_owned().to_owned());
                match parts.get(1) {
                    Some(value) => {
                        self.queue.borrow_mut().replace(value.to_owned().to_owned());
                        Ok(())
                    }
                    None => Err(format!("QueueName.split_ | '{}' does not have structure 'Service.queue'", self.name)),
                }
            }
            None => Err(format!("QueueName.split_ | '{}' does not have structure 'Service.queue'", self.name)),
        }
    }
    ///
    /// Returns splitted Service and queue names
    pub fn split(&self) -> Result<(String, String), String> {
        if self.service.borrow().is_none() || self.queue.borrow().is_none() {
            if let Err(err) = self.split_() {
                return Err(err);
            };
        }
        match self.service.borrow().clone() {
            Some(service) => {
                match self.queue.borrow().clone() {
                    Some(queue) => {
                        Ok((service, queue))
                    }
                    None => Err(format!("QueueName.split | Part 'queue' not found in the '{}'", self.name)),
                }
            }
            None => Err(format!("QueueName.split | Part 'service' not found in the '{}'", self.name)),
        }
    }
    ///
    /// Returns the Service name
    pub fn service(&self) -> Result<String, String> {
        if self.service.borrow().is_none() {
            if let Err(err) = self.split_() {
                return Err(err);
            };
        }
        match self.service.borrow().clone() {
            Some(service) => Ok(service),
            None => Err(format!("QueueName.service | Part 'service' not found in the '{}'", self.name)),
        }
    }
    ///
    /// Returns the Service's queue name
    pub fn queue(&self) -> Result<String, String> {
        if self.queue.borrow().is_none() {
            if let Err(err) = self.split_() {
                return Err(err);
            };
        }
        match self.queue.borrow().clone() {
            Some(queue) => Ok(queue),
            None => Err(format!("QueueName.queue | Part 'queue' not found in the '{}'", self.name)),
        }
    }
    ///
    /// Try to split, panics if split() returns err
    pub fn validate(self) -> Self {
        if let Err(err) = self.split() {
            panic!("QueueName.validate | Error: '{}'", err)
        }
        self
    }
}
//
//
impl std::fmt::Display for QueueName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
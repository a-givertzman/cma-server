use std::{cell::RefCell, fmt::{Debug, Display}};
use concat_string::concat_string;
use log::error;
///
/// Creates Point name from parts, dividing parts with single "/" character
/// - path1 + "" = /path1
/// - path1 + path2 = /path1/path2
/// - path1/ + path2 = /path1/path2
/// - path1 + /path2 = /path1/path2
/// - path1/ + /path2 = /path1/path2
#[derive(Clone)]
pub struct Name {
    parent: String,
    me: String,
    joined: RefCell<Option<String>>,
}
//
// 
impl Name {
    ///
    /// Creates new instance of Name from 'parent' and 'me'
    pub fn new(parent: impl Into<String>, me: impl Into<String>) -> Self {
        Self {
            parent: parent.into(),
            me: me.into(),
            joined: RefCell::new(None),
        }
    }
    ///
    /// Returns 
    pub fn join(&self) -> String {
        if self.joined.borrow().is_none() {
            let parent = match self.parent.chars().next() {
                Some(parent_first) => {
                    if parent_first == '/' {
                        self.parent.clone()
                    } else {
                        concat_string!("/", self.parent)
                    }
                }
                None => {
                    "/".to_owned()
                }
            };
            *self.joined.borrow_mut() = Some(match parent.chars().last() {
                Some(parent_last) => {
                    match self.me.chars().next() {
                        Some(me_first) => {
                            if parent_last == '/' && me_first == '/' {
                                concat_string!(parent, self.me[1..])
                            } else if parent_last == '/' && me_first != '/' {
                                concat_string!(parent, self.me)
                            } else if parent_last != '/' && me_first == '/' {
                                concat_string!(parent, self.me)
                            } else {
                                concat_string!(parent, "/", self.me)
                            }
                        }
                        None => {
                            parent.to_owned()
                        }
                    }
                }
                None => {
                    match self.me.chars().next() {
                        Some(me_first) => {
                            if me_first == '/' {
                                self.me.clone()
                            } else {
                                concat_string!("/", self.me)
                            }
                        }
                        None => {
                            panic!("PointName.new | Parent or name must not be empty")
                        }
                    }
                }
            });
        }
        self.joined.borrow().clone().unwrap_or_else(|| {
            error!("PointName.new | Join error for parent: '{}', me: '{}'", self.parent, self.me);
            String::new()
        })
    }
    ///
    /// Returns original parent
    pub fn parent(&self) -> String {
        self.parent.clone()
    }
    ///
    /// Returns original me
    pub fn me(&self) -> String {
        self.me.clone()
    }
}
//
// 
impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.join())
    }
}
//
// 
impl Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.join())
    }
}
// ///
// /// 
impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.join() == other.join()
    }
}
//
// 
impl From<&Name> for String {
    fn from(value: &Name) -> Self {
        value.join()
    }
}
//
// 
impl From<Name> for String {
    fn from(value: Name) -> Self {
        value.join()
    }
}
//
// 
impl From<String> for Name {
    fn from(value: String) -> Self {
        Name { parent: value, me: String::new(), joined: RefCell::new(None) }
    }
}
//
// 
impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name { parent: value.to_owned(), me: String::new(), joined: RefCell::new(None) }
    }
}
///
/// 
unsafe impl Sync for Name {}

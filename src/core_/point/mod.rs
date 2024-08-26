//!
//! #### Point - The Entity of the information
//! 
//! The Entity of the information. Contains fallowing:
//! - name
//! - type
//! - value
//! - status
//! - cot
//! - timestamp
//!
//! <details>
//! 
//! ##### Point.name
//! 
//! Unique within all the system (similar to the linux system full file path).
//! 
//! - Begins with "/",
//! - consists of the path divided by the "/",
//! - Ends with the name (name can be divided by the dot / multiple dots)
//! 
//! Examples:
//! 
//! ```js
//! '/AppName/Service/Point.Name'
//! '/AppName/Device/Point.Name'
//! '/AppName/SubAppName/Device/Point.Name'
//! ```
//! 
//! ##### Point.type
//! 
//! The type of the containing information stored in the Point.value field. Fallowing types are supported:
//! 
//! - Bool - true / false
//! - Int - i64 - The 64-bit signed integer type.
//! - Real - f32 - A 32-bit floating point type (specifically, the "binary32" type defined in IEEE 754-2008).
//! - Double - f64 - A 64-bit floating point type (specifically, the "binary64" type defined in IEEE 754-2008).
//! - String - string of the variable length
//! 
//! ##### Point.value
//! 
//! Contains the information of the type corresponding with the Point.type field
//! 
//! ##### Point.status
//! 
//! The status of the containing information:
//! 
//! - Ok = 0 - Information was successfully updated from the source device;
//! - Obsolete = 2 - For example system was jast started and information stored from the prevouse session;
//! - TimeInvalid = 3 - The time of the server / Device is not synchronized with precision time source;
//! - Invalid = 10 - Information was read from the device but currently connection with that device is lost;
//! 
//! ##### Point.cot
//! 
//! Cause and direction of the transmission:
//! 
//! - Inf - Information - common information basically comming from the Device / Server to the Client
//! - Act - Activation - the command comming from the Client to the Device / Server
//! - ActCon - Activation | Confirmation - the confirmation of the successfully executed command
//! - ActErr - Activation | Error - the information about falied command
//! - Req - Request - the request to the server, besicaly contains some specific json
//! - ReqCon - Request | Confirmation reply - the confirmation of the successfully performed request
//! - ReqErr - Request | Error reply - the information about falied request
//! 
//! ##### Point.timestamp
//! 
//! Contains a timestamp in the format corresponding with RFC 3339 and ISO 8601 date and time string:
//! 
//! - Includes milliseconds and microseconds,
//! - Local time zone offset can be included
//! 
//! Such as:
//! `2024-02-19T12:16:57.648504907Z`
//! 
//! </details>
pub mod point_hlr;

pub mod point_type;

pub mod point_tx_id;
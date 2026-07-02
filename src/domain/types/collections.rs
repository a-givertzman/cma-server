///
/// DashMap with simple & fast hasher
///  - This hashing algorithm should not be used for cryptographic, or in scenarios where DOS attacks are a concern.
pub(crate) type FxDashMap<K, V> = dashmap::DashMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
pub(crate) type FxSccHashMap<K, V> = scc::HashMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

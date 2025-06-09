///
/// DashMap with simple & fast hasher
///  - This hashing algorithm should not be used for cryptographic, or in scenarios where DOS attacks are a concern.
pub type FxDashMap<K, V> = dashmap::DashMap<K, V, std::hash::BuildHasherDefault<hashers::fx_hash::FxHasher>>;

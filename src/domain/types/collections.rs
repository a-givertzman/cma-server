/// HashMap with simple & fast hasher
///  - This hashing algorithm should not be used for cryptographic, or in scenarios where DOS attacks are a concern.
pub(crate) type FxHashMap<K, V> = std::collections::HashMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
// Создаем трейт для нового метода конструктора
pub(crate) trait FxHashMapExt {
    fn with_capacity(capacity: usize) -> Self;
}
impl<K, V> FxHashMapExt for FxHashMap<K, V> {
    fn with_capacity(capacity: usize) -> Self {
        std::collections::HashMap::with_capacity_and_hasher(capacity, std::hash::BuildHasherDefault::default())
    }
}
pub(crate) type FxIndexMap<K, V> = indexmap::IndexMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
// Создаем трейт для нового метода конструктора
pub(crate) trait FxIndexMapExt {
    fn with_capacity(capacity: usize) -> Self;
}
impl<K, V> FxIndexMapExt for FxIndexMap<K, V> {
    fn with_capacity(capacity: usize) -> Self {
        indexmap::IndexMap::with_capacity_and_hasher(capacity, std::hash::BuildHasherDefault::default())
    }
}
///
/// DashMap with simple & fast hasher
///  - This hashing algorithm should not be used for cryptographic, or in scenarios where DOS attacks are a concern.
pub(crate) type FxDashMap<K, V> = dashmap::DashMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
pub(crate) type FxSccHashMap<K, V> = scc::HashMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

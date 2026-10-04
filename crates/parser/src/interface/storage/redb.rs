/// Custom error type for storage-related errors, encompassing both database errors from `redb` and serialization errors from `serde_json`.
#[derive(Debug)]
pub enum StorageError {
    Redb(Box<dyn std::error::Error + Send + Sync>),
    Json(serde_json::Error),
}

/// Implementations to convert various `redb` error types into `StorageError`, allowing for unified error handling in the `EncounterStore` methods.
impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Redb(e) => write!(f, "Database error: {e}"),
            StorageError::Json(e) => write!(f, "Serialization error: {e}"),
        }
    }
}

/// Implement the standard `Error` trait for `StorageError` to allow it to be used with the `?` operator and other error handling utilities in Rust.
impl std::error::Error for StorageError {}

/// Implement conversions from `serde_json::Error` and various `redb` error types to `StorageError`, enabling seamless error handling in the `EncounterStore` methods.
impl From<serde_json::Error> for StorageError {
    fn from(e: serde_json::Error) -> Self {
        StorageError::Json(e)
    }
}

/// Macro to implement `From` conversions for multiple `redb` error types into `StorageError::Redb`,
/// allowing for concise and consistent error handling in the `EncounterStore` methods.
macro_rules! impl_from_redb_error {
    ($($t:ty),*) => {
        $(impl From<$t> for StorageError {
            fn from(e: $t) -> Self {
                StorageError::Redb(Box::new(e))
            }
        })*
    };
}

// Implement `From` conversions for the various `redb` error types into `StorageError::Redb` using the `impl_from_redb_error` macro.
impl_from_redb_error!(
    redb::DatabaseError,
    redb::TransactionError,
    redb::TableError,
    redb::CommitError,
    redb::StorageError
);

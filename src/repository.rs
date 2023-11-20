use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;

/// Represents a share entry in the database.
///
/// This struct is used to store and retrieve share entries, which include a share and the sender's information.
///
/// # Fields
///
/// * `share` - A tuple containing the share identifier (u8) and the share data (Vec<u8>).
/// * `sender` - A vector of bytes representing the sender's information.
///
/// # Examples
///
/// Creating a new `ShareEntry`:
///
/// ```rust
/// use mpcnet::repository::ShareEntry;
///
/// let share_entry = ShareEntry {
///     share: (1, vec![2, 3, 4]),
///     sender: vec![5, 6, 7],
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShareEntry {
    pub share: (u8, Vec<u8>),
    pub sender: Vec<u8>,
}

/// Defines the Data Access Object (DAO) trait for `ShareEntry`.
///
/// This trait specifies the methods for inserting, retrieving, updating, and deleting `ShareEntry` objects
/// in a data store.
pub trait ShareEntryDaoTrait: Send + Sync {
    /// Inserts a `ShareEntry` into the data store.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the `ShareEntry`.
    /// * `entry` - The `ShareEntry` to be inserted.
    ///
    /// # Returns
    ///
    /// A `Result` indicating the success or failure of the operation.
    fn insert(&self, key: &str, entry: &ShareEntry) -> Result<(), Box<dyn Error>>;

    /// Retrieves a `ShareEntry` from the data store by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the `ShareEntry` to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<ShareEntry>`. `None` if the key does not exist.
    fn get(&self, key: &str) -> Result<Option<ShareEntry>, Box<dyn Error>>;

    fn get_all(&self) -> Result<Vec<(String, ShareEntry)>, Box<dyn Error>>;

    /// Updates an existing `ShareEntry` in the data store.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the `ShareEntry` to update.
    /// * `entry` - The new `ShareEntry` data.
    ///
    /// # Returns
    ///
    /// A `Result` indicating the success or failure of the operation.
    fn update(&self, key: &str, entry: &ShareEntry) -> Result<(), Box<dyn Error>>;

    /// Deletes a `ShareEntry` from the data store by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the `ShareEntry` to delete.
    ///
    /// # Returns
    ///
    /// A `Result` indicating the success or failure of the operation.
    fn delete(&self, key: &str) -> Result<(), Box<dyn Error>>;
}

/// A `ShareEntryDaoTrait` implementation using Sled, an embedded database.
///
/// This struct provides methods to interact with the Sled database for operations on `ShareEntry` objects.
///
/// # Fields
///
/// * `db` - The Sled database instance.
pub struct SledShareEntryDao {
    db: Db,
}

impl SledShareEntryDao {
    /// Creates a new instance of `SledShareEntryDao`.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The path to the sled database.
    ///
    /// # Returns
    ///
    /// A `Result` containing `SledShareEntryDao` or an error.
    ///
    /// # Examples
    ///
    /// Creating a new instance:
    ///
    /// ```ignore
    /// use mpcnet::repository::SledShareEntryDao;
    ///
    /// let dao = SledShareEntryDao::new("path/to/db").unwrap();
    /// ```
    pub fn new(db_path: &str) -> Result<Self, Box<dyn Error>> {
        let db = sled::open(db_path)?;
        Ok(SledShareEntryDao { db })
    }
}

impl ShareEntryDaoTrait for SledShareEntryDao {
    /// Inserts a new `ShareEntry` into the Sled database.
    ///
    /// This method serializes the `ShareEntry` into a JSON string and stores it in the database under the provided key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key under which to store the entry.
    /// * `entry` - The `ShareEntry` to be stored.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mpcnet::repository::ShareEntry;
    /// use mpcnet::repository::SledShareEntryDao;
    /// use mpcnet::repository::ShareEntryDaoTrait;
    ///
    /// let dao = SledShareEntryDao::new("path/to/db").unwrap();
    /// let entry = ShareEntry { share: (1, vec![1, 2, 3]), sender: vec![4, 5, 6] };
    /// dao.insert("some_key", &entry);
    /// ```
    fn insert(&self, key: &str, entry: &ShareEntry) -> Result<(), Box<dyn Error>> {
        let serialized = serde_json::to_string(entry)?;
        self.db.insert(key, serialized.as_bytes())?;
        Ok(())
    }

    /// Retrieves a `ShareEntry` from the Sled database by its key.
    ///
    /// If the key exists, the method deserializes the stored JSON string back into a `ShareEntry`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the entry to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing `Option<ShareEntry>`. `None` if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mpcnet::repository::ShareEntryDaoTrait;
    /// use mpcnet::repository::SledShareEntryDao;
    ///
    /// let dao = SledShareEntryDao::new("path/to/db").unwrap();
    /// let entry = dao.get("some_key").unwrap();
    /// ```
    fn get(&self, key: &str) -> Result<Option<ShareEntry>, Box<dyn Error>> {
        if let Some(found) = self.db.get(key)? {
            let entry: ShareEntry = serde_json::from_slice(&found)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    fn get_all(&self) -> Result<Vec<(String, ShareEntry)>, Box<dyn Error>> {
        let mut entries = Vec::new();
        for entry in self.db.iter() {
            let (key, value) = entry?;
            let entry: ShareEntry = serde_json::from_slice(&value)?;
            entries.push((String::from_utf8(key.to_vec())?, entry));
        }
        Ok(entries)
    }

    /// Updates an existing `ShareEntry` in the Sled database.
    ///
    /// This method essentially re-inserts the entry, replacing the old one.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the entry to update.
    /// * `entry` - The new `ShareEntry`.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mpcnet::repository::ShareEntry;
    /// use mpcnet::repository::SledShareEntryDao;
    /// use mpcnet::repository::ShareEntryDaoTrait;
    ///
    /// let dao = SledShareEntryDao::new("path/to/db").unwrap();
    /// let new_entry = ShareEntry { share: (1, vec![7, 8, 9]), sender: vec![10, 11, 12] };
    /// dao.update("some_key", &new_entry).unwrap();
    /// ```
    fn update(&self, key: &str, entry: &ShareEntry) -> Result<(), Box<dyn Error>> {
        self.insert(key, entry)
    }

    /// Deletes a `ShareEntry` from the Sled database by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the entry to delete.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mpcnet::repository::ShareEntryDaoTrait;
    /// use mpcnet::repository::SledShareEntryDao;
    ///
    /// let dao = SledShareEntryDao::new("path/to/db").unwrap();
    /// dao.delete("some_key");
    /// ```
    fn delete(&self, key: &str) -> Result<(), Box<dyn Error>> {
        self.db.remove(key)?;
        Ok(())
    }
}

pub struct HashMapShareEntryDao {
    pub map: Mutex<HashMap<String, ShareEntry>>,
}

impl ShareEntryDaoTrait for HashMapShareEntryDao {
    /// Inserts a new `ShareEntry` into the HashMap.
    ///
    /// This method locks the HashMap for thread safety and inserts the provided entry.
    ///
    /// # Arguments
    ///
    /// * `key` - The key under which to store the entry.
    /// * `entry` - The `ShareEntry` to be stored.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mpcnet::repository::ShareEntry;
    /// use std::collections::HashMap;
    /// use std::sync::Mutex;
    /// use mpcnet::repository::HashMapShareEntryDao;
    /// use mpcnet::repository::ShareEntryDaoTrait;
    ///
    /// let dao = HashMapShareEntryDao { map: Mutex::new(HashMap::new()) };
    /// let entry = ShareEntry { share: (1, vec![1, 2, 3]), sender: vec![4, 5, 6] };
    /// dao.insert("some_key", &entry).unwrap();
    /// ```
    fn insert(&self, key: &str, entry: &ShareEntry) -> Result<(), Box<dyn Error>> {
        let mut map = self.map.lock().unwrap();
        map.insert(key.to_string(), entry.clone());
        Ok(())
    }

    /// Retrieves a `ShareEntry` from the HashMap by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the entry to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing `Option<ShareEntry>`. `None` if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mpcnet::repository::{ShareEntry, ShareEntryDaoTrait, HashMapShareEntryDao};
    /// use std::collections::HashMap;
    /// use std::sync::Mutex;
    ///
    /// let dao = HashMapShareEntryDao { map: Mutex::new(HashMap::new()) };
    /// let entry = dao.get("some_key").unwrap();
    /// ```
    fn get(&self, key: &str) -> Result<Option<ShareEntry>, Box<dyn Error>> {
        let map = self.map.lock().unwrap();
        Ok(map.get(key).cloned())
    }

    fn get_all(&self) -> Result<Vec<(String, ShareEntry)>, Box<dyn Error>> {
        let map = self.map.lock().unwrap();
        let mut entries = Vec::new();
        for (key, value) in map.iter() {
            entries.push((key.clone(), value.clone()));
        }
        Ok(entries)
    }

    /// Updates an existing `ShareEntry` in the HashMap.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the entry to update.
    /// * `entry` - The new `ShareEntry`.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the key does not exist in the HashMap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mpcnet::repository::{ShareEntry, ShareEntryDaoTrait, HashMapShareEntryDao};
    /// use std::collections::HashMap;
    /// use std::sync::Mutex;
    ///
    /// let dao = HashMapShareEntryDao { map: Mutex::new(HashMap::new()) };
    /// let new_entry = ShareEntry { share: (1, vec![7, 8, 9]), sender: vec![10, 11, 12] };
    /// dao.update("some_key", &new_entry);
    /// ```
    fn update(&self, key: &str, entry: &ShareEntry) -> Result<(), Box<dyn Error>> {
        let mut map = self.map.lock().unwrap();
        if map.contains_key(key) {
            map.insert(key.to_string(), entry.clone());
            Ok(())
        } else {
            Err("Key not found".into())
        }
    }

    /// Deletes a `ShareEntry` from the HashMap by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the entry to delete.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mpcnet::repository::{ShareEntry, ShareEntryDaoTrait, HashMapShareEntryDao};
    /// use std::collections::HashMap;
    /// use std::sync::Mutex;
    ///
    /// let dao = HashMapShareEntryDao { map: Mutex::new(HashMap::new()) };
    /// dao.delete("some_key").unwrap();
    /// ```
    fn delete(&self, key: &str) -> Result<(), Box<dyn Error>> {
        let mut map = self.map.lock().unwrap();
        map.remove(key);
        Ok(())
    }
}

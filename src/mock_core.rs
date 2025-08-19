//! Mock implementations for testing without hft-core dependency

use std::sync::atomic::{AtomicU64, Ordering};

static FREQUENCY_MHZ: AtomicU64 = AtomicU64::new(3000); // Default 3GHz

pub fn cpu_frequency_mhz() -> u64 {
    FREQUENCY_MHZ.load(Ordering::Relaxed)
}

pub fn set_cpu_frequency_mhz(freq: u64) {
    FREQUENCY_MHZ.store(freq, Ordering::Relaxed);
}

/// Mock timestamp type for benchmarking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn now() -> Self {
        Self(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64)
    }
    
    pub fn as_nanos(&self) -> u64 {
        self.0
    }
}

/// Mock price type for benchmarking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Price(i64);

impl Price {
    pub fn new(value: f64) -> Self {
        Self((value * 100.0) as i64) // Simple fixed-point
    }
    
    pub fn as_f64(&self) -> f64 {
        self.0 as f64 / 100.0
    }
}

impl std::ops::Add for Price {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Price {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

/// Mock quantity type for benchmarking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Quantity(u64);

impl Quantity {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::ops::Add for Quantity {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Quantity {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

/// Mock SPSC ring buffer for benchmarking
pub struct SPSCRingBuffer<T> {
    buffer: Vec<Option<T>>,
    head: std::sync::atomic::AtomicUsize,
    tail: std::sync::atomic::AtomicUsize,
    capacity: usize,
}

impl<T> SPSCRingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity + 1);
        for _ in 0..=capacity {
            buffer.push(None);
        }
        
        Self {
            buffer,
            head: std::sync::atomic::AtomicUsize::new(0),
            tail: std::sync::atomic::AtomicUsize::new(0),
            capacity: capacity + 1,
        }
    }
    
    pub fn push(&self, item: T) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let next_head = (head + 1) % self.capacity;
        
        if next_head == self.tail.load(Ordering::Acquire) {
            false // Full
        } else {
            unsafe {
                let ptr = self.buffer.as_ptr().add(head) as *mut Option<T>;
                ptr.write(Some(item));
            }
            self.head.store(next_head, Ordering::Release);
            true
        }
    }
    
    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Acquire);
        if tail == self.head.load(Ordering::Acquire) {
            None // Empty
        } else {
            let item = unsafe {
                let ptr = self.buffer.as_ptr().add(tail) as *mut Option<T>;
                ptr.read()
            };
            self.tail.store((tail + 1) % self.capacity, Ordering::Release);
            item
        }
    }
}

unsafe impl<T: Send> Send for SPSCRingBuffer<T> {}
unsafe impl<T: Send> Sync for SPSCRingBuffer<T> {}

/// Mock wait-free hash table for benchmarking  
pub struct WaitFreeHashTable<K, V> {
    buckets: Vec<std::sync::RwLock<Vec<(K, V)>>>,
    capacity: usize,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> WaitFreeHashTable<K, V> {
    pub fn new(capacity: usize) -> Self {
        let mut buckets = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buckets.push(std::sync::RwLock::new(Vec::new()));
        }
        
        Self { buckets, capacity }
    }
    
    fn hash(&self, key: &K) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.capacity as u64) as usize
    }
    
    pub fn insert(&self, key: K, value: V) -> bool {
        let bucket_idx = self.hash(&key);
        let mut bucket = self.buckets[bucket_idx].write().unwrap();
        
        // Update existing or insert new
        for (k, v) in bucket.iter_mut() {
            if k == &key {
                *v = value;
                return false; // Updated existing
            }
        }
        
        bucket.push((key, value));
        true // Inserted new
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        let bucket_idx = self.hash(key);
        let bucket = self.buckets[bucket_idx].read().unwrap();
        
        bucket.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }
}

/// Mock object pool for benchmarking
pub struct ObjectPool<T> {
    objects: std::sync::Mutex<Vec<T>>,
}

impl<T> Default for ObjectPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ObjectPool<T> {
    pub fn new() -> Self {
        Self {
            objects: std::sync::Mutex::new(Vec::new()),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            objects: std::sync::Mutex::new(Vec::with_capacity(capacity)),
        }
    }
    
    pub fn get(&self, create_fn: impl FnOnce() -> T) -> T {
        if let Ok(mut objects) = self.objects.lock() {
            objects.pop().unwrap_or_else(create_fn)
        } else {
            create_fn()
        }
    }
    
    pub fn put(&self, obj: T) {
        if let Ok(mut objects) = self.objects.lock() {
            objects.push(obj);
        }
    }
}

/// Mock NUMA arena allocator for benchmarking
pub struct NumaArenaAllocator {
    node_id: usize,
}

impl NumaArenaAllocator {
    pub fn new(node_id: usize) -> Self {
        Self { node_id }
    }
    
    pub fn allocate(&self, size: usize) -> Vec<u8> {
        // Just use regular allocation for the mock
        vec![0u8; size]
    }
    
    pub fn node_id(&self) -> usize {
        self.node_id
    }
}
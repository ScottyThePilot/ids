use std::collections::hash_map::{self, HashMap, RandomState};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::iter::FusedIterator;
use std::ops::Index;
use std::cmp::Ordering;
use std::fmt;

const ORDERING: AtomicOrdering = AtomicOrdering::SeqCst;



/// A context for spawning unique IDs.
/// The type parameter is to allow you to specify a "family" so that
/// IDs can be impossible to mix up, as they may cause issues if you
/// "cross the beams" so to speak. This is optional though, and it
/// defaults to the unit type, `()` as the default family.
#[derive(Debug)]
pub struct IdContext<F = ()> {
  current_id: u64,
  family: PhantomData<F>
}

impl<F> IdContext<F> {
  #[cfg(feature = "serde")]
  const fn with_current_id(current_id: u64) -> IdContext<F> {
    IdContext { current_id, family: PhantomData }
  }

  /// Creates a new ID context.
  pub const fn new() -> IdContext<F> {
    IdContext {
      current_id: 0,
      family: PhantomData
    }
  }

  /// Spawns the next unique ID for this context.
  pub fn next_id(&mut self) -> Id<F> {
    let id = self.current_id;
    self.current_id += 1;
    Id { id, family: PhantomData }
  }

  fn duplicate(&self) -> IdContext<F> {
    IdContext {
      current_id: self.current_id,
      family: PhantomData
    }
  }
}

impl<F> Default for IdContext<F> {
  #[inline]
  fn default() -> IdContext<F> {
    IdContext::new()
  }
}



pub struct Id<F = ()> {
  id: u64,
  family: PhantomData<F>
}

impl<F> fmt::Debug for Id<F> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_tuple("Id")
      .field(&self.id)
      .finish()
  }
}

impl<F> Clone for Id<F> {
  #[inline]
  fn clone(&self) -> Self {
    Id { id: self.id.clone(), family: PhantomData }
  }
}

impl<F> Copy for Id<F> {}

impl<F> PartialEq for Id<F> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl<F> Eq for Id<F> {}

impl<F> PartialOrd for Id<F> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    PartialOrd::partial_cmp(&self.id, &other.id)
  }
}

impl<F> Ord for Id<F> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    Ord::cmp(&self.id, &other.id)
  }
}

impl<F> Hash for Id<F> {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.id.hash(state)
  }
}

#[cfg(feature = "serde")]
impl<F> serde::Serialize for Id<F> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    serializer.serialize_newtype_struct("Id", &self.id)
  }
}

#[cfg(feature = "serde")]
impl<'de, F> serde::Deserialize<'de> for Id<F> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    struct IdVisitor<F>(PhantomData<F>);

    impl<'de, F> serde::de::Visitor<'de> for IdVisitor<F> {
      type Value = Id<F>;

      fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("tuple struct Id or u64")
      }

      #[inline]
      fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
      where D: serde::Deserializer<'de>, {
        Ok(Id {
          id: <u64 as serde::Deserialize>::deserialize(deserializer)?,
          family: PhantomData
        })
      }

      #[inline]
      fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
      where A: serde::de::SeqAccess<'de>, {
        match seq.next_element::<u64>()? {
          Some(id) => Ok(Id { id, family: PhantomData }),
          None => Err(serde::de::Error::invalid_length(0, &"tuple struct Id with 1 element"))
        }
      }

      #[inline]
      fn visit_u64<E>(self, id: u64) -> Result<Self::Value, E>
      where E: serde::de::Error {
        Ok(Id { id, family: PhantomData })
      }
    }

    deserializer.deserialize_newtype_struct("Id", IdVisitor(PhantomData))
  }
}



/// An atomic ID context.
/// This is just like `IdContext`, but operates atomically
/// and can be shared between threads.
#[derive(Debug)]
pub struct AtomicIdContext<F = ()> {
  current_id: AtomicU64,
  family: PhantomData<F>
}

impl<F> AtomicIdContext<F> {
  pub const fn new() -> AtomicIdContext<F> {
    AtomicIdContext {
      current_id: AtomicU64::new(0),
      family: PhantomData
    }
  }

  pub fn next_id(&self) -> Id<F> {
    let id = self.current_id.fetch_add(1, ORDERING);
    Id { id, family: PhantomData }
  }
}

impl<F> Default for AtomicIdContext<F> {
  #[inline]
  fn default() -> AtomicIdContext<F> {
    AtomicIdContext::new()
  }
}



pub struct AtomicId<F = ()> {
  id: u64,
  family: PhantomData<F>
}

impl<F> fmt::Debug for AtomicId<F> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_tuple("AtomicId")
      .field(&self.id)
      .finish()
  }
}

impl<F> Clone for AtomicId<F> {
  #[inline]
  fn clone(&self) -> Self {
    AtomicId { id: self.id.clone(), family: PhantomData }
  }
}

impl<F> Copy for AtomicId<F> {}

impl<F> PartialEq for AtomicId<F> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl<F> Eq for AtomicId<F> {}

impl<F> PartialOrd for AtomicId<F> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    PartialOrd::partial_cmp(&self.id, &other.id)
  }
}

impl<F> Ord for AtomicId<F> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    Ord::cmp(&self.id, &other.id)
  }
}

impl<F> Hash for AtomicId<F> {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.id.hash(state)
  }
}



/// A hashmap with `Id<T>`s as opaque keys.
pub struct IdMap<T, S = RandomState> {
  context: IdContext<T>,
  map: HashMap<Id<T>, T, S>
}

impl<T> IdMap<T, RandomState> {
  pub fn new() -> IdMap<T, RandomState> {
    IdMap::from_map_raw(HashMap::new())
  }

  pub fn with_capacity(capacity: usize) -> IdMap<T, RandomState> {
    IdMap::from_map_raw(HashMap::with_capacity(capacity))
  }
}

impl<T, S> IdMap<T, S> {
  #[inline]
  fn from_map_raw(map: HashMap<Id<T>, T, S>) -> IdMap<T, S> {
    IdMap { context: IdContext::new(), map }
  }

  #[inline]
  pub fn with_hasher(hash_builder: S) -> IdMap<T, S> {
    IdMap::from_map_raw(HashMap::with_hasher(hash_builder))
  }

  #[inline]
  pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> IdMap<T, S> {
    IdMap::from_map_raw(HashMap::with_capacity_and_hasher(capacity, hash_builder))
  }

  #[inline]
  pub fn capacity(&self) -> usize {
    self.map.capacity()
  }

  #[inline]
  pub fn ids(&self) -> Ids<'_, T> {
    Ids { inner: self.map.keys() }
  }

  #[inline]
  pub fn values(&self) -> Values<'_, T> {
    Values { inner: self.map.values() }
  }

  #[inline]
  pub fn values_mut(&mut self) -> ValuesMut<'_, T> {
    ValuesMut { inner: self.map.values_mut() }
  }

  #[inline]
  pub fn iter(&self) -> Iter<'_, T> {
    Iter { inner: self.map.iter() }
  }

  #[inline]
  pub fn iter_mut(&mut self) -> IterMut<'_, T> {
    IterMut { inner: self.map.iter_mut() }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.map.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

  #[inline]
  pub fn drain(&mut self) -> Drain<'_, T> {
    self.context.current_id = 0;
    Drain { inner: self.map.drain() }
  }

  #[inline]
  pub fn clear(&mut self) {
    self.context.current_id = 0;
    self.map.clear();
  }

  #[inline]
  pub fn hasher(&self) -> &S {
    self.map.hasher()
  }
}

impl<T, S: BuildHasher> IdMap<T, S> {
  #[inline]
  pub fn reserve(&mut self, additional: usize) {
    self.map.reserve(additional);
  }

  #[inline]
  pub fn shrink_to_fit(&mut self) {
    self.map.shrink_to_fit()
  }

  // TODO: Entry

  #[inline]
  pub fn get(&self, id: Id<T>) -> Option<&T> {
    self.map.get(&id)
  }

  #[inline]
  pub fn contains_id(&self, id: Id<T>) -> bool {
    self.map.contains_key(&id)
  }

  #[inline]
  pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
    self.map.get_mut(&id)
  }

  pub fn insert_new(&mut self, value: T) -> Id<T> {
    let id = self.context.next_id();
    let result = self.map.insert(id, value);
    debug_assert!(result.is_none());
    id
  }

  #[inline]
  pub fn insert(&mut self, id: Id<T>, value: T) -> Option<T> {
    self.map.insert(id, value)
  }

  #[inline]
  pub fn remove(&mut self, id: Id<T>) -> Option<T> {
    self.map.remove(&id)
  }

  #[inline]
  pub fn retain<F>(&mut self, mut f: F)
  where F: FnMut(Id<T>, &mut T) -> bool {
    self.map.retain(move |&id, v| f(id, v));
  }
}

impl<T: fmt::Debug, S> fmt::Debug for IdMap<T, S> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_map().entries(self.iter()).finish()
  }
}

impl<T: Clone, S: Clone> Clone for IdMap<T, S> {
  fn clone(&self) -> Self {
    IdMap {
      context: self.context.duplicate(),
      map: self.map.clone()
    }
  }
}

impl<T, S: Default> Default for IdMap<T, S> {
  #[inline]
  fn default() -> Self {
    IdMap::with_hasher(Default::default())
  }
}

impl<T, S: BuildHasher> Index<Id<T>> for IdMap<T, S> {
  type Output = T;

  #[inline]
  fn index(&self, id: Id<T>) -> &Self::Output {
    self.get(id).expect("no entry found for id")
  }
}

#[cfg(feature = "serde")]
impl<T, H> serde::Serialize for IdMap<T, H>
where T: serde::Serialize, H: BuildHasher {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    self.map.serialize(serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, T, H> serde::Deserialize<'de> for IdMap<T, H>
where T: serde::Deserialize<'de>, H: BuildHasher + Default {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    struct IdMapVisitor<T, H>(PhantomData<IdMap<T, H>>);

    impl<'de, T, H> serde::de::Visitor<'de> for IdMapVisitor<T, H>
    where T: serde::Deserialize<'de>, H: BuildHasher + Default {
      type Value = IdMap<T, H>;

      fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a map")
      }

      fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
      where A: serde::de::MapAccess<'de> {
        let size = serde::__private::size_hint::cautious(map.size_hint());
        let mut values = HashMap::with_capacity_and_hasher(size, H::default());
        let mut current_id: u64 = 0;

        while let Some((key, value)) = map.next_entry::<Id<T>, T>()? {
          current_id = current_id.max(key.id);
          values.insert(key, value);
        };

        current_id += 1;

        Ok(IdMap {
          context: IdContext::with_current_id(current_id),
          map: values
        })
      }
    }

    deserializer.deserialize_newtype_struct("Id", IdMapVisitor(PhantomData))
  }
}



macro_rules! impl_iterator_struct {
  ($Iter:ident, $Item:ty) => {
    impl_iterator_struct!(@ $Iter, $Item, s -> s.inner.next());
  };
  ($Iter:ident, $Item:ty, $map:expr) => {
    impl_iterator_struct!(@ $Iter, $Item, s -> s.inner.next().map($map));
  };
  (@ $Iter:ident, $Item:ty, $self:ident -> $next:expr) => {
    impl<T: fmt::Debug> fmt::Debug for $Iter<'_, T> {
      #[inline]
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
      }
    }

    impl<'a, T> Iterator for $Iter<'a, T> {
      type Item = $Item;

      #[inline]
      fn next(&mut self) -> Option<$Item> {
        let $self = self;
        $next
      }

      #[inline]
      fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
      }
    }

    impl<T> ExactSizeIterator for $Iter<'_, T> {
      #[inline]
      fn len(&self) -> usize {
        self.inner.len()
      }
    }

    impl<T> FusedIterator for $Iter<'_, T> {}
  };
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Ids<'a, T: 'a> {
  inner: hash_map::Keys<'a, Id<T>, T>
}

impl_iterator_struct!(Ids, Id<T>, |id| *id);

#[repr(transparent)]
#[derive(Clone)]
pub struct Values<'a, T: 'a> {
  inner: hash_map::Values<'a, Id<T>, T>
}

impl_iterator_struct!(Values, &'a T);

#[repr(transparent)]
pub struct ValuesMut<'a, T: 'a> {
  inner: hash_map::ValuesMut<'a, Id<T>, T>
}

impl_iterator_struct!(ValuesMut, &'a mut T);

#[repr(transparent)]
#[derive(Clone)]
pub struct Iter<'a, T: 'a> {
  inner: hash_map::Iter<'a, Id<T>, T>
}

impl_iterator_struct!(Iter, (Id<T>, &'a T), |(id, v)| (*id, v));

#[repr(transparent)]
pub struct IterMut<'a, T: 'a> {
  inner: hash_map::IterMut<'a, Id<T>, T>
}

impl_iterator_struct!(IterMut, (Id<T>, &'a mut T), |(id, v)| (*id, v));

#[repr(transparent)]
pub struct Drain<'a, T: 'a> {
  inner: hash_map::Drain<'a, Id<T>, T>
}

impl_iterator_struct!(Drain, T, |(_, v)| v);

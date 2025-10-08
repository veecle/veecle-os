# Reader/Writer Variants And API Design

## Risks

Ready-to-use actors that users can just bring into their application are at the heart of veecle-os.
Breaking changes to the actor API, and by extension reader/writer API, fracture the actor ecosystem.
By defining the basic reader and writer types early and building features on top, previous versions remain compatible even if they might not utilize the latest features.

If we can build everything on top of the single-/multi-reader/writer by using abstractions like `Queued<T>`, we are in a position to expand our functionality in a non-breaking way and everything build on the initial implementation continues to work.

`Queued<T>` is a struct to use as slot value to enable a slot to contain an ordered list of values.
Can be implemented using a `Queued<T: Storable>` type within a slot.
Convenience methods can be implemented using a `IsQueue` trait that's implemented for `Queued<T>` and `impl<T: Storable + IsQueue + 'static> Reader<...>{...}`.

## Slot Value State

The state of a value within a slot describes whether the value has been seen by a reader or has been published by a writer.

### Writer Perspective

A value can either be "published" or "unpublished".
Published means that readers will have or already had a chance to read the value.
This is the case when the future (actor) containing the writer has yielded control back to the executor.
Due to executor semantics, all readers now have a chance to read the value.

Unpublished means that readers have not yet had a chance to read the value.
Every value written by a writer remains unpublished until the future (actor) yields control to the executor.

### Reader Perspective

A value can either be "seen" or "unseen".
A seen value is a value that has been read or marked as seen after it has been published.
An unseen value is a value that has not been read or marked as seen yet.

## Types Of Readers and Writers

### InitializedReader

The `InitializedReader` allows actors to wait until a value has been published.
This can lead to deadlock situations where actor `A` has an initialized reader for value `X` and a writer for value `Y`, while actor `B` has an initialized reader for value `Y` and the writer for value `X`.
Neither can make progress since writes to `X` and `Y` are blocked.
While the same situation is possible without the `InitializedReader`, the `InitializedReader` makes it less obvious why the deadlock occurred compared to explicit waiting for a value to be published.

With the removal of the `InitializedReader`, users lose the ability to do immediate reads without the wrapping `Option`.
By adding a `async fn read_unseen<U>(&self, f: impl FnOnce(&T::DataType) -> U) -> U ` method to the non-initialized reader, users are still able to avoid the wrapping `Option` in the typical case.

## Reader API

```rust
impl<T> Reader<'_, T>
where
  T: Storable + 'static,
{
  /// Reads the current value and marks it seen, if it was unseen before.
  fn read<U>(&self, f: impl FnOnce(Option<&T::DataType>) -> U) -> U {}

  /// `wait_for_unseen` + `read`
  async fn read_unseen<U>(&self, f: impl FnOnce(&T::DataType) -> U) -> U {}

  /// read` + `clone`
  fn read_cloned(&self) -> Option<T::DataType>
  where
    T::DataType: Clone,
  {}

  /// `wait_for_unseen` + `read_cloned`
  async fn read_unseen_cloned(&self) -> T::DataType
  where
    T::DataType: Clone,
  {}

  /// Waits until the writer publishes a new value.
  async fn wait_for_unseen(&mut self) -> &mut Self {}

  /// Checks whether a value has been published since the slot has been last read/marked seen.
  fn is_unseen(&self) -> bool {}

  /// Marks a value as "seen".
  fn mark_seen(&self) {}
}
```

## ExclusiveReader API

```rust
/// Same as Reader, plus the methods listed here.
impl<T> ExclusiveReader<'_, T>
where
  T: Storable + 'static,
{
  /// Takes the current value out of the slot.
  fn take<U>(&mut self) -> Option<U::DataType> {}

  /// `wait_for_unseen` + `take`
  async fn take_unseen<U>(&mut self) -> U::DataType {}
}
```

## Writer API

```rust
impl<T> Writer<'_, T>
where
  T: Storable + 'static,
{
  /// Reads the current value.
  fn read<U>(&self, f: impl FnOnce(Option<&T::DataType>) -> U) -> U {}

  /// Overwrites the current value even if it hasn't been published yet.
  /// Marks the value as `unseen` for readers.
  /// Will require documentation to explain that a value that has been written but not explicitly published
  /// will be considered published at the next await point that requires the future to yield.
  fn write(&mut self, item: T::DataType) {}

  /// Publishes the current value.
  /// Does not mark the value as `unseen` for readers.
  async fn publish(&mut self) {}

  /// `write` + `publish`
  async fn write_and_publish(&mut self, item: T::DataType) {}

  /// Modifies the current value.
  /// Marks the value as `unseen` for readers.
  fn modify(&mut self, f: impl FnOnce(&mut Option<T::DataType>)) {}

  /// `modify` + `publish`
  async fn modify_and_publish(&mut self, f: impl FnOnce(&mut Option<T::DataType>)) {}

  /// Queries whether a value has been read by all associated readers.
  /// Can be used to check for back-pressure.
  /// A single slow/non-reading reader can cause this to never be true.
  fn has_been_seen(&self) -> bool {}
}
```

## Multi-Reader & Multi-Writer

### Multi-Writer

A multi-writer can be combined with `N` single-readers.
Every reader can be exclusive.
The mapping between value space within the slot and a reader is fixed but not deterministic between compilations.

### Multi-Reader

A multi-reader can be combined with either `N` single-writers.
The mapping between value space within the slot and a writer is fixed but not deterministic between compilations.

### Wiring Up Different Numbers Of Reader/Writers

It is currently not possible to determine the amount of readers/writers at compiletime or on the type-level.
This prevents implementations that automatically set up the correct amount of value spaces for a multi-reader/writer.

`Multi{Reader/Writer}<T, const N: usize>` requires every actor to be generic over the amount of values in a slot.

By requiring the user to specify `N`, we side-step the issue of automatically detecting how many reader/writers there are for a multi-reader/writer at compile time.
We can verify whether the user-supplied `N` value is correct at runtime and issue an error as we do to detect orphan readers/writers already.

This approach allows the user to use static memory that will be guaranteed to fit all elements for caching or further processing.
Since this "leaks" into the actor signature, it makes actor declarations more verbose.

#### Access To Values

##### Const Generics

`read<..., const I: usize>(...) -> ...` would allow statically checked access to entries.
This approach doesn't allow users to build actors that can deal with different numbers of value spaces.

#### `Vec`-Like

Basically the same approach as a `Vec`, the user isn't aware of how many value spaces there are at compiletime.
Provides fallible methods like `get(index: usize)-> Option<Option<T::DataType>>`.
This requires double-wrapping in `Option` to distinguish between the value space not existing and an empty value space.

#### Iterators

Iterators provide non-fallible access to all or a subset of value spaces without requiring user to explicitly deal with the number of entries.

#### Conclusion

Iterators fit well with the read all or read unseen access pattern of readers.
For writers, providing an iterator of to-be-published values to the write method can allow producing values only if there is space in the writer.

## Multi-Reader API

A multi-reader is automatically an exclusive reader.

```rust
impl<T, const N: usize> MultiReader<'_, T, N>
where
  T: Storable + 'static,
{
  /// Reads all current values.
  /// Will skip value spaces that don't currently have a value.
  /// Marks every unseen value as seen.
  fn read_all<U>(&self, f: impl FnOnce(ValueIter<&T>) -> U) -> U {}

  /// Reads all unseen values.
  /// Marks every unseen value as seen.
  fn read_unseen<U>(&self, f: impl FnOnce(UnseenIter<&T>) -> U) -> U {}

  /// Takes all current values.
  /// Will skip value spaces that don't currently have a value.
  /// Marks every unseen value as seen.
  fn take_all<U>(&self, f: impl FnOnce(ValueIter<T>) -> U) -> U {}

  /// Takes all unseen values.
  /// Marks every unseen value as seen.
  fn take_unseen<U>(&self, f: impl FnOnce(UnseenIter<T>) -> U) -> U {}

  /// Waits until at least one writer publishes a new value.
  async fn wait_for_any_unseen(&mut self) -> &mut Self {}

  /// Waits until every writer published a new value.
  async fn wait_for_all_unseen(&mut self) -> &mut Self {}

  /// Checks whether any value has been published since the slot has been last read/marked seen.
  fn has_any_unseen(&self) -> bool {}

  /// Checks whether all values have been published since the slot has been last read/marked seen.
  fn has_all_unseen(&self) -> bool {}

  /// Marks all values as "seen".
  fn mark_all_seen(&self) {}
}
```

## Multi-Writer API

```rust
impl<T, const N: usize> MultiWriter<'_, T, N>
where
  T: Storable + 'static,
{
  /// Reads all current values.
  /// Will skip value spaces that don't currently have a value.
  fn read_all<U>(&self, f: impl FnOnce(ValueIter<&T>) -> U) -> U {}

  /// Overwrites current values, even if they haven't been published yet.
  /// Marks all values written as `unseen` for readers.
  /// Will require documentation to explain that a value that has been written but not explicitly published
  /// will be considered published at the next await point that requires the future to yield.
  fn write<U>(&self, item_iter: impl Iterator<Item=T::DataType>) {}

  /// Publishes all current values.
  /// Does not mark the values as `unseen` for readers.
  async fn publish(&mut self) {}

  /// `write` + `publish`
  async fn write_and_publish(&mut self, item_iter: impl Iterator<Item=T::DataType>) {}

  /// Modifies all current values, skips value spaces currently unoccupied
  /// Marks all currently occupied values as `unseen` for readers.
  fn modify(&mut self, f: impl FnOnce(Option<&mut T::DataType>)) {}

  /// `modify` + `publish`
  async fn modify_and_publish(&mut self, f: impl FnOnce(Option<&mut T::DataType>)) {}

  /// Queries whether any value has been read by the associated reader.
  /// Can be used to check for back-pressure.
  fn has_any_been_seen(&self) -> bool {}

  /// Queries whether all values have been read by the associated readers.
  /// Can be used to check for back-pressure.
  /// A single slow reader can cause this to never be true.
  fn have_all_been_seen(&self) -> bool {}
}
```

## Available Combinations

- 1 Single-Writer : 1 Single-Reader (Reader can be exclusive)
- 1 Single-Writer : N Single-Reader (Readers cannot be exclusive)
- 1 Multi-Writer : N Single-Reader (Readers can be exclusive)
- N Single-Writer : 1 Multi-Reader (Reader is automatically exclusive)

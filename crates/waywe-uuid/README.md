# type-uuid

This crate provides a way to specify a stable, unique identifier for Rust types.

## Assigning UUIDs to Types

This crate provides the [`TypeUuid`] trait, which defines a single const item
`UUID`. This value is a byte array containing the raw bytes of the UUID for the
type.

You will have to manually specify the UUID for any type implementing
[`TypeUuid`], but this crate provides a custom derive to make that easy to do:

```rust
use type_uuid::TypeUuid;

#[derive(TypeUuid)]
#[uuid = "d4adfc76-f5f4-40b0-8e28-8a51a12f5e46"]
struct MyType;
```

While the derive handles the tedious work of converting the UUID into a byte
array suitable for use with the [`TypeUuid`] trait, you'll still need to
generate a valid UUID in order to assign it to your type. To do so, we
recommend using https://www.uuidtools.com/generate/v4, which provides a quick way
generate new UUIDs that you can paste into your code.

[`TypeUuid`]: ./trait.TypeUuid.html

# Rust Runtime — Developer Reference

> **Emitter**: `/home/ytr/Specodec/typespec-emitter-rust/src/index.ts`

---

## 1. Type Mapping Table

| TypeSpec Type | Rust Type | Notes |
|---|---|---|
| `string` | `String` | |
| `boolean` | `bool` | |
| `int8` | `i8` | Explicit sized types |
| `int16` | `i16` | |
| `int32` | `i32` | |
| `int64` | `i64` | |
| `uint8` | `u8` | |
| `uint16` | `u16` | |
| `uint32` | `u32` | |
| `uint64` | `u64` | |
| `float32` | `f32` | |
| `float64`, `float`, `decimal` | `f64` | |
| `bytes` | `Vec<u8>` | |
| `integer` | `i32` | |
| Enum | native `enum` with `From<i32>` impl | |
| Array `<T>` | `Vec<T>` | |
| Record `<V>` | `std::collections::HashMap<String, V>` | |
| Model | `struct` with `#[derive(Debug, Clone, Default)]` | |
| Union | `pub enum` with data variants | |

---

## 2. Model Representation

Models are Rust structs with standard derives:

```rust
#[derive(Debug, Clone, Default)]
pub struct MyModel {
    pub name: String,
    pub age: i32,
    pub tags: Vec<String>,
}
```

All fields are `pub`. `Default` is derived for zero-initialization in decode.

---

## 3. Optional / Nullable

- Optional fields use `Option<T>`.
- **Self-referencing detection**: The emitter has a `needsBox()` function. If a model references itself (directly or indirectly), `Option<T>` becomes `Option<Box<T>>` to satisfy Rust's sizing requirements:
  ```rust
  pub struct TreeNode {
      pub value: i32,
      pub left: Option<Box<TreeNode>>,
      pub right: Option<Box<TreeNode>>,
  }
  ```

---

## 4. Union Representation

Discriminated unions use Rust `enum` with data variants:

```rust
pub enum MyUnion {
    VariantA(i32),
    VariantB(String),
    Undefined,
}
```

Encode emits the `_tag` string and variant data as separate object fields. Decode reads `_tag`, then dispatches to the appropriate variant constructor.

---

## 5. Enum Representation

Native Rust `enum` with `From<i32>` implementation using **unsafe transmute**:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
}

impl From<i32> for Color {
    fn from(v: i32) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}
```

The enum numeric value is written as `i32` in msgpack, and as a string in JSON/gron (via the enum member name).

---

## 6. Ryu Implementation

- **Bit extraction**: `f.to_bits()` (native method on `f32`/`f64`).
- **`mul_shift_64`**: Uses Rust's native **`u128`** type — much cleaner than Go/Dart:
  ```rust
  let b0 = (m as u128) * (mul[0] as u128);
  let b2 = (m as u128) * (mul[1] as u128);
  let b0_hi = b0 >> 64;
  let sum_val = b0_hi + b2;
  ((sum_val >> (shift - 64)) as u64) & 0xFFFFFFFFFFFFFFFF
  ```
- **Tables**: `const` arrays with `u64` values. f32: static `[u64]`; f64: static `[[u64; 2]]` (array of `[u64; 2]`).
- **`multiple_of_power_of_5_64`**: Uses `wrapping_mul(5)` in a loop instead of `pow()` for overflow-safe computation.
- **`decimalLength9`/`decimalLength17`**: Unsigned integer comparisons on `u32`/`u64`.
- **Output format**: Scientific notation.

---

## 7. MsgPack Reader/Writer

**Reader** (`MsgPackReader`):
- Accumulates over `&[u8]` with `pos: usize` cursor.
- Reads integers via byte-level assembly (`u32::from_be_bytes`, etc.).
- `read_float32`: reads 4 bytes → `f32::from_be_bytes(...)`.
- `read_float64`: reads 8 bytes → `f64::from_be_bytes(...)`.
- `read_int64`: reads two u32 and combines with sign handling.
- `container_count: Vec<u32>` for map/array nesting.
- All methods return `Result<T, SCodecError>` with `?` operator.

**Writer** (`MsgPackWriter`):
- Accumulates into `Vec<u8>`.
- `write_float32`: uses `f.to_bits().to_be_bytes()`.
- `write_float64`: uses `f.to_bits().to_be_bytes()`.
- String/write operations use `extend_from_slice`.

---

## 8. JSON Reader/Writer

**Reader** (`JsonReader`):
- Works on `String` (decoded from `&[u8]` via `std::str::from_utf8`).
- Parses `\uXXXX` with **surrogate pair support** (same algorithm as baseline).
- `read_int32`/`read_uint32`: uses `str::parse::<i32>()` etc.
- `read_int64`/`read_uint64`: supports quoted string parsing.
- NaN: `f64::NAN`; Infinity: `f64::INFINITY` / `f64::NEG_INFINITY`.
- `read_float32`: parses as `f64` then casts to `f32`.
- All operations return `Result<_, SCodecError>`.

**Writer** (`JsonWriter`):
- Accumulates into `Vec<u8>` directly (bytes, not strings).
- Escape: byte-level matching on control characters; emits `\u00XX` for `0x00..=0x1F`.
- NaN/Infinity: quoted `"NaN"`, `"Infinity"`, `"-Infinity"`.
- `int64`/`uint64`: formatted as quoted strings.
- Base64: manual encoding using const `CHARS` array.
- Uses `format_float32`/`format_float64` for non-special floats.

---

## 9. Gron Reader/Writer

**Reader** (`GronReader`):
- Parses `path = value;` lines; uses `String::from_utf8_lossy` for tolerant UTF-8 conversion.
- Context stack: `Vec<CtxInfo>` (struct with `prefix`, `ctx_type`, `index`).
- `unescape`: handles `\uXXXX` via `u32::from_str_radix(hex, 16)` → `char::from_u32` — **supports surrogate pairs** via `char::from_u32`.
- `read_bytes`: Base64 decode via `base64` library call.
- `is_null`: compares raw value to `"null"`.

**Writer** (`GronWriter`):
- Accumulates into `Vec<u8>` bytes directly.
- Path starts with `"json"`, with nesting tracked by `segments: Vec<String>` + `nesting: Vec<NestInfo>`.
- `int64`/`uint64`: quoted decimal strings.
- NaN/Infinity: quoted strings.

---

## 10. State Management

- **Mutable** struct-based state via `&mut self` receivers.
- Error propagation uses `Result<T, SCodecError>` with the `?` operator throughout.
- `SCodecError` is a simple struct with a `message: String` field, implementing `std::error::Error`.
- `SpecCodec<T>` is a `Copy + Clone` struct holding function pointers (not trait objects).

---

## 11. SpecReader / SpecWriter Interfaces

### SpecReader

```rust
pub trait SpecReader {
    fn begin_object(&mut self) -> Result<(), SCodecError>;
    fn has_next_field(&mut self) -> Result<bool, SCodecError>;
    fn read_field_name(&mut self) -> Result<String, SCodecError>;
    fn end_object(&mut self) -> Result<(), SCodecError>;
    fn begin_array(&mut self) -> Result<(), SCodecError>;
    fn has_next_element(&mut self) -> Result<bool, SCodecError>;
    fn end_array(&mut self) -> Result<(), SCodecError>;
    fn read_string(&mut self) -> Result<String, SCodecError>;
    fn read_bool(&mut self) -> Result<bool, SCodecError>;
    fn read_int32(&mut self) -> Result<i32, SCodecError>;
    fn read_int64(&mut self) -> Result<i64, SCodecError>;
    fn read_uint32(&mut self) -> Result<u32, SCodecError>;
    fn read_uint64(&mut self) -> Result<u64, SCodecError>;
    fn read_float32(&mut self) -> Result<f32, SCodecError>;
    fn read_float64(&mut self) -> Result<f64, SCodecError>;
    fn read_null(&mut self) -> Result<(), SCodecError>;
    fn read_bytes(&mut self) -> Result<Vec<u8>, SCodecError>;
    fn read_enum(&mut self) -> Result<String, SCodecError>;
    fn is_null(&mut self) -> Result<bool, SCodecError>;
    fn skip(&mut self) -> Result<(), SCodecError>;
}
```

### SpecWriter

```rust
pub trait SpecWriter {
    fn write_string(&mut self, value: &str);
    fn write_bool(&mut self, value: bool);
    fn write_int32(&mut self, value: i32);
    fn write_int64(&mut self, value: i64);
    fn write_uint32(&mut self, value: u32);
    fn write_uint64(&mut self, value: u64);
    fn write_float32(&mut self, value: f32);
    fn write_float64(&mut self, value: f64);
    fn write_null(&mut self);
    fn write_bytes(&mut self, value: &[u8]);
    fn write_enum(&mut self, value: &str);
    fn begin_object(&mut self, field_count: usize);
    fn write_field(&mut self, name: &str);
    fn end_object(&mut self);
    fn begin_array(&mut self, element_count: usize);
    fn next_element(&mut self);
    fn end_array(&mut self);
    fn to_bytes(&self) -> Vec<u8>;
}
```

Key difference: Writer methods are **infallible** (no `Result`), while Reader methods all return `Result`.

---

## 12. Emitter Generation Pattern

### Model encode
```rust
pub fn encode_my_model(w: &mut dyn SpecWriter, obj: &MyModel) {
    w.begin_object(2);
    w.write_field("name");
    w.write_string(&obj.name);
    w.write_field("age");
    w.write_int32(obj.age);
    w.end_object();
}
```

### Model decode
```rust
pub fn decode_my_model(r: &mut dyn SpecReader) -> Result<MyModel, SCodecError> {
    r.begin_object()?;
    let mut obj = MyModel::default();
    while r.has_next_field()? {
        match r.read_field_name()?.as_str() {
            "name" => obj.name = r.read_string()?,
            "age" => obj.age = r.read_int32()?,
            _ => { r.skip()?; }
        }
    }
    r.end_object()?;
    Ok(obj)
}
```

All decode returns `Result<T, SCodecError>` with `?` for error propagation. Writer encode is infallible.

---

## 13. Known Quirks / Bugs

- **Gron unescape**: Rust's `char::from_u32` correctly handles surrogate pairs (unlike Dart/Go/TS gron readers).
- **Writer infallible**: Writer methods do not return `Result` — buffer operations cannot fail in Rust (Vec can only fail on allocation, which panics).
- **`SpecUndefined`**: Simple unit struct `pub struct SpecUndefined;` with `#[derive(Clone, Debug)]` — no singleton pattern.
- **Enum decode**: Uses `unsafe { std::mem::transmute(v) }` for `i32` → enum conversion. If the emitter guarantees valid values, this is safe. Invalid wire data causes undefined behavior.
- **`FormatRegistry::register`**: Consumes `self` and returns `Self` (builder pattern, not in-place mutation).
- **`Dispatch`**: `respond` takes `&T` (reference), `dispatch` returns `Result<T, SCodecError>` (owned). `encode` expects `&T` (borrow).

---

## 14. DevContainer

- **Base image**: `dev:all`
- **Tooling**: Rust via `mise` shims + `/root/.cargo/bin` in PATH; `CARGO_HOME=/root/.cargo`
- **Build**: Copies `Cargo.toml` first, creates stub `lib.rs`, runs `cargo check` (dependency resolution only), then copies `src/` and runs `cargo check` again (full type check). Uses `--mount=type=cache` for `/root/.cargo/registry` and `/app/target`.
- **Output** (`FROM scratch`): copies `/app/Cargo.toml` to `/out/Cargo.toml`

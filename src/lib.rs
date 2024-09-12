/*!

This crate provides simple traits for reading and writing objects.

The traits [`ObjectReader`] and [`ObjectWriter`] are **not** intended as a generalized serialization
framework like [serde](https://serde.rs), they are provided to simply read/write specific object
types in specific formats. These traits were refactored from the `rdftk_io` crate that provided a
number of parsers and generators for different RDF representations.

As a number of implementations require options to configure parsers and generators the trait
[`HasOptions`] can be implemented to provide this in a common manner.

# Example Writer

3. The type `TestObject` is the type we wich to be able to write, it has a single string field.
1. The type `TestError` is required as it can be created from an IO error instance.
2. The type `TestOptions` are the options that configure our generator, currently only defining the
   amount of indentation before writing `TestObject` instances.
4. The type TestWriter` is where the magic starts, ...
   1. It contains a field for the options above.
   2. It implements the trait `HasOptions` using the macro `impl_has_options!`.
   3. It implements the trait `ObjectWriter`.
5. We the construct an example instance of the test object and an instance of the writer with
   options.
6. Finally, we write the example object and compare it to expected results.

```rust
use objio::{impl_has_options, HasOptions, ObjectWriter};
use std::io::Write;

#[derive(Debug, Default)]
struct TestObject { // our writeable type
    value: String,
}
# impl From<&str> for TestObject {
#     fn from(s: &str) -> Self {
#         Self { value: s.to_string() }
#     }
# }

#[derive(Debug, Default)]
struct TestError {} // implements From<std::io::Error>
# impl From<::std::io::Error> for TestError {
#    fn from(_: ::std::io::Error) -> Self {
#        Self {}
#    }
# }

#[derive(Debug, Default)]
struct TestOptions {
    indent: usize,
}

#[derive(Debug, Default)]
struct TestWriter {
    options: TestOptions,
}

impl_has_options!(TestWriter, TestOptions);

impl ObjectWriter<TestObject> for TestWriter {
    type Error = TestError;

    fn write<W>(&self, w: &mut W, object: &TestObject) -> Result<(), Self::Error>
    where
        W: Write,
    {
        let indent = self.options.indent;
        let value = &object.value;
        w.write_all(format!("{:indent$}{value}", "").as_bytes())?;
        Ok(())
    }
}

let example = TestObject::from("Hello");
let writer = TestWriter::default().with_options(TestOptions { indent: 2 });

assert_eq!(
    writer.write_to_string(&example).unwrap(),
    "  Hello".to_string()
);

```

*/

#![warn(
    unknown_lints,
    // ---------- Stylistic
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    macro_use_extern_crate,
    nonstandard_style, /* group */
    noop_method_call,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    // ---------- Future
    future_incompatible, /* group */
    rust_2021_compatibility, /* group */
    // ---------- Public
    missing_debug_implementations,
    // missing_docs,
    unreachable_pub,
    // ---------- Unsafe
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    // ---------- Unused
    unused, /* group */
)]
#![deny(
    // ---------- Public
    exported_private_dependencies,
    // ---------- Deprecated
    anonymous_parameters,
    bare_trait_objects,
    ellipsis_inclusive_range_patterns,
    // ---------- Unsafe
    deref_nullptr,
    drop_bounds,
    dyn_drop,
)]

use std::fs::OpenOptions;
use std::io::{Cursor, Read, Write};
use std::path::Path;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// This trait is implemented by reader or writer types to attach option instances for
/// configuration.
///
/// Note that the option type `T` **must** implement `Default` so that it is not necessary to
/// require an option during construction of the reader or writer.
///
pub trait HasOptions<T: Default> {
    ///
    /// A builder-like function that can be called after the default constructor.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use objio::{impl_has_options, HasOptions};
    /// # fn get_options_from_config() -> TestOptions { Default::default() }
    /// #[derive(Debug, Default)]
    /// struct TestOptions {
    ///     indent: usize,
    /// }
    ///
    /// #[derive(Debug, Default)]
    /// struct TestWriter {
    ///     options: TestOptions,
    /// }
    ///
    /// impl_has_options!(TestWriter, TestOptions);
    ///
    /// let writer = TestWriter::default()
    ///     .with_options(get_options_from_config());
    /// ```
    ///
    fn with_options(self, options: T) -> Self
    where
        Self: Sized,
    {
        let mut self_mut = self;
        self_mut.set_options(options);
        self_mut
    }

    ///
    /// Set the current options value to `options`.
    ///
    fn set_options(&mut self, options: T);

    ///
    /// Returns a reference to the current options.
    ///
    fn options(&self) -> &T;
}

// ------------------------------------------------------------------------------------------------

///
/// The trait implemented by types which read instances of `T`.
///
pub trait ObjectReader<T> {
    ///
    /// The type indicating errors, this **must** implement the conversion from `io::Error` as this
    /// error is intrinsic to the methods on `Read`. This constraint allows the error type to also
    /// signal parser errors related to the content itself.
    ///
    type Error: From<::std::io::Error>;

    ///
    /// Read an instance of `T` from the provided implementation of `Read`.
    ///
    fn read<R>(&self, r: &mut R) -> Result<T, Self::Error>
    where
        R: Read;

    ///
    /// Read an instance of `T` from the provided string.
    ///
    fn read_from_string<S>(&self, string: S) -> Result<T, Self::Error>
    where
        S: AsRef<str>,
    {
        let mut data = string.as_ref().as_bytes();
        self.read(&mut data)
    }

    ///
    /// Read an instance of `T` from the file identified by `path`.
    ///
    /// This method will return an IO error if the path is invalid, or file does not exist.
    ///
    fn read_from_file<P>(&self, path: P) -> Result<T, Self::Error>
    where
        P: AsRef<Path>,
    {
        let mut file = OpenOptions::new().read(true).open(path.as_ref())?;
        self.read(&mut file)
    }
}

// ------------------------------------------------------------------------------------------------

///
/// The trait implemented by types which write instances of `T`.
///
pub trait ObjectWriter<T> {
    ///
    /// The type indicating errors, this **must** implement the conversion from `io::Error` as this
    /// error is intrinsic to the methods on `Write`. This constraint allows the error type to also
    /// signal serialization errors related to the content itself.
    ///
    type Error: From<::std::io::Error>;

    ///
    /// Write an instance of `T` to the provided implementation of `Write`.
    ///
    fn write<W>(&self, w: &mut W, object: &T) -> Result<(), Self::Error>
    where
        W: Write;

    ///
    /// Write an instance of `T` to, and return, a string.
    ///
    fn write_to_string(&self, object: &T) -> Result<String, Self::Error> {
        let mut buffer = Cursor::new(Vec::new());
        self.write(&mut buffer, object)?;
        Ok(String::from_utf8(buffer.into_inner()).unwrap())
    }

    ///
    /// Write an instance of `T` into the file identified by `path`.
    ///
    /// This method will return an IO error if the path is invalid, or the file is not writeable.
    /// If the file exists it will be replaced.
    ///
    fn write_to_file<P>(&self, object: &T, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path>,
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.as_ref())?;
        self.write(&mut file, object)
    }
}

// ------------------------------------------------------------------------------------------------
// Public Macros
// ------------------------------------------------------------------------------------------------

///
/// Provides a boiler-place implementation of [`HasOptions`].
///
#[macro_export]
macro_rules! impl_has_options {
    ($impl_type: ty, $option_type: ty) => {
        impl_has_options!($impl_type, $option_type, options);
    };
    ($impl_type: ty, $option_type: ty, $field_name: ident) => {
        impl $crate::HasOptions<$option_type> for $impl_type {
            fn set_options(&mut self, options: $option_type) {
                self.$field_name = options;
            }

            fn options(&self) -> &$option_type {
                &self.$field_name
            }
        }
    };
}

///
/// Provides a simple implementation of [`ObjectWriter`] where the existing implementation of
/// `Display` provides the serialized form via the `ToString` trait.
///
#[macro_export]
macro_rules! impl_to_string_writer {
    ($impl_type: ty, $object_type: ty) => {
        impl $crate::ObjectWriter<$object_type> for $impl_type {
            fn write<W>(&self, w: &mut W, object: &$object_type) -> Result<()>
            where
                W: Write,
            {
                let stringified: String =
                    <$object_type as ::std::string::ToString>::to_string(object);
                w.write_all(stringified.as_bytes()).map_err(Error::Io)?;
                Ok(())
            }
        }
    };
}

///
/// Provides a simple implementation of [`ObjectWriter`] where an existing implementation of
/// `Into<String>` provides the serialized form.
///
#[macro_export]
macro_rules! impl_into_string_writer {
    ($impl_type: ty, $object_type: ty) => {
        impl $crate::ObjectWriter<$object_type> for $impl_type {
            fn write<W>(&self, w: &mut W, object: &$object_type) -> Result<()>
            where
                W: Write,
            {
                let stringified: String =
                    <$object_type as ::std::convert::Into<::std::string::String>>::into(object);
                w.write_all(stringified.as_bytes()).map_err(Error::Io)?;
                Ok(())
            }
        }
    };
}

// ------------------------------------------------------------------------------------------------
// Unit Tests
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct TestError {}

    impl From<::std::io::Error> for TestError {
        fn from(_: ::std::io::Error) -> Self {
            Self {}
        }
    }

    #[test]
    fn test_manual_options() {
        #[derive(Debug, Default)]
        struct TestOptions {
            count: u32,
        }

        #[derive(Debug, Default)]
        struct TestObject {
            options: TestOptions,
        }

        impl HasOptions<TestOptions> for TestObject {
            fn set_options(&mut self, options: TestOptions) {
                self.options = options
            }

            fn options(&self) -> &TestOptions {
                &self.options
            }
        }

        let obj = TestObject::default().with_options(TestOptions { count: 2 });

        assert_eq!(obj.options().count, 2);
    }

    #[test]
    fn test_macro_options() {
        #[derive(Debug, Default)]
        struct TestOptions {
            count: u32,
        }

        #[derive(Debug, Default)]
        struct TestObject {
            options: TestOptions,
        }

        impl_has_options!(TestObject, TestOptions);

        let obj = TestObject::default().with_options(TestOptions { count: 2 });

        assert_eq!(obj.options().count, 2);
    }

    #[test]
    fn test_writer_to_string() {
        #[derive(Debug, Default)]
        struct TestObject {}

        #[derive(Debug, Default)]
        struct TestWriter {}

        impl ObjectWriter<TestObject> for TestWriter {
            type Error = TestError;

            fn write<W>(&self, w: &mut W, _object: &TestObject) -> Result<(), Self::Error>
            where
                W: Write,
            {
                w.write_all(b"Hello")?;
                Ok(())
            }
        }

        let writer = TestWriter::default();

        assert_eq!(
            writer.write_to_string(&TestObject::default()).unwrap(),
            "Hello".to_string()
        );
    }
}

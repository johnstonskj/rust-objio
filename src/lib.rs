/*!

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

pub trait HasOptions<T: Default> {
    fn with_options(self, options: T) -> Self
    where
        Self: Sized,
    {
        let mut self_mut = self;
        self_mut.set_options(options);
        self_mut
    }

    fn set_options(&mut self, options: T);

    fn options(&self) -> &T;
}

// ------------------------------------------------------------------------------------------------

pub trait ObjectReader<T> {
    type Error: From<::std::io::Error>;

    fn read<R>(&self, r: &mut R) -> Result<T, Self::Error>
    where
        R: Read;

    fn read_from_string<S>(&self, string: S) -> Result<T, Self::Error>
    where
        S: AsRef<str>,
    {
        let mut data = string.as_ref().as_bytes();
        self.read(&mut data)
    }

    fn read_from_file<P>(&self, path: P) -> Result<T, Self::Error>
    where
        P: AsRef<Path>,
    {
        let mut file = OpenOptions::new().read(true).open(path.as_ref())?;
        self.read(&mut file)
    }
}

// ------------------------------------------------------------------------------------------------

pub trait ObjectWriter<T> {
    type Error: From<::std::io::Error>;

    fn write<W>(&self, w: &mut W, object: &T) -> Result<(), Self::Error>
    where
        W: Write;

    fn write_to_string(&self, object: &T) -> Result<String, Self::Error> {
        let mut buffer = Cursor::new(Vec::new());
        self.write(&mut buffer, object)?;
        Ok(String::from_utf8(buffer.into_inner()).unwrap())
    }

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

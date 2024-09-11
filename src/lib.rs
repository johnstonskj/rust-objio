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

use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{Cursor, Read, Write};
use std::path::Path;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum Error<T: ::std::error::Error> {
    Io(::std::io::Error),
    Other(T),
}

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

pub trait ObjectReader<T> {
    type Error: ::std::error::Error;

    fn read<R>(&self, r: &mut R) -> Result<T, Error<Self::Error>>
    where
        R: Read;

    fn read_from_string<S>(&self, string: S) -> Result<T, Error<Self::Error>>
    where
        S: AsRef<str>,
    {
        let mut data = string.as_ref().as_bytes();
        self.read(&mut data)
    }

    fn read_from_file<P>(&self, path: P) -> Result<T, Error<Self::Error>>
    where
        P: AsRef<Path>,
    {
        let mut file = OpenOptions::new()
            .read(true)
            .open(path.as_ref())
            .map_err(Error::Io)?;
        self.read(&mut file)
    }
}

pub trait ObjectWriter<T> {
    type Error: ::std::error::Error;

    fn write<W>(&self, w: &mut W, object: &T) -> Result<(), Error<Self::Error>>
    where
        W: Write;

    fn write_to_string(&self, object: &T) -> Result<String, Error<Self::Error>> {
        let mut buffer = Cursor::new(Vec::new());
        self.write(&mut buffer, object)?;
        Ok(String::from_utf8(buffer.into_inner()).unwrap())
    }

    fn write_to_file<P>(&self, object: &T, path: P) -> Result<(), Error<Self::Error>>
    where
        P: AsRef<Path>,
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.as_ref())
            .map_err(Error::Io)?;
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
// Implementations
// ------------------------------------------------------------------------------------------------

impl<T: ::std::error::Error> Display for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Io(e) => format!("I/O error: {e}"),
                Self::Other(e) => format!("Non-I/O error: {e}"),
            }
        )
    }
}

impl<T: ::std::error::Error + 'static> ::std::error::Error for Error<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Other(e) => Some(e),
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------

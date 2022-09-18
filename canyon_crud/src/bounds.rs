use std::any::Any;
use std::{borrow::Cow};

use canyon_connection::tiberius::ColumnData;

use canyon_connection::{tokio_postgres::types::ToSql, tiberius::IntoSql};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

/// Created for retrieve the field's name of a field of a struct, giving 
/// the Canoyn's autogenerated enum with the variants that maps this
/// fields.
/// 
/// ```
/// pub struct Struct<'a> {
///     pub some_field: &'a str
/// }
/// 
/// // Autogenerated enum
/// #[derive(Debug)]
/// #[allow(non_camel_case_types)]
/// pub enum StructField {
///     some_field
/// }
/// ```
/// So, to retrieve the field's name, something like this w'd be used on some part
/// of the Canyon's Manager crate, to wire the necessary code to pass the field
/// name, retrieved from the enum variant, to a called.
/// 
/// // Something like:
/// `let struct_field_name_from_variant = StructField::some_field.field_name_as_str();`
pub trait FieldIdentifier {
    fn field_name_as_str(self) -> String;
}

/// Represents some kind of introspection to make the implementors
/// retrieves a value inside some variant of an associated enum type.
/// and convert it to an [`String`], to enable the convertion of 
/// that value into something that can be part of an SQL query.
/// 
/// It's a generification to convert everything to a string representation
/// in SQL syntax, so the clauses can use any value to make filters
/// 
/// Ex:
/// `SELECT * FROM some_table WHERE id = '2'`
/// 
/// That '2' it's extracted from some enum that implements [`FieldValueIdentifier`],
/// where usually the variant w'd be something like:
/// 
/// ```
/// pub enum Enum {
///     IntVariant(i32)
/// }
/// ```
/// so, the `.value(self)` method it's called over `self`, gets the value for that variant
/// (or another specified in the logic) and returns that value as an [`String`]
pub trait FieldValueIdentifier {
    fn value(self) -> String;
}

impl FieldValueIdentifier for &str {
    fn value(self) -> String {
        self.to_string()
    }
}

/// Bounds to some type T in order to make it callable over some fn parameter T
/// 
/// Represents the ability of an struct to be considered as candidate to perform
/// actions over it as it holds the 'parent' side of a foreign key relation.
/// 
/// Usually, it's used on the Canyon macros to retrieve the column that 
/// this side of the relation it's representing
pub trait ForeignKeyable {
    /// Retrieves the field related to the column passed in
    fn get_fk_column(&self, column: &str) -> Option<String>;
}


/// To define trait objects that helps to relates the necessary bounds in the 'IN` SQL clause
pub trait InClauseValues: ToSql + ToString {}

/// Defines a trait to join types that can represent
/// PrimaryKey type.
/// 
/// Canyon only accepts values of i32, i64 
/// + any Rust String type that can work 
/// as any Rust string variation.
pub trait PrimaryKey<'a>: IntoSql<'a> + ToSql + Sync + Send + Clone + QueryParameters<'a> {}

impl<'a> PrimaryKey<'a> for i32 {}
impl<'a> PrimaryKey<'a> for i64 {}
// impl<'a> PrimaryKey<'a> for &'a str {}
impl<'a> PrimaryKey<'a> for String {}
// impl<'a> PrimaryKey<'a> for &String {}


// TODO IMPLEMENT THE OPTIONALS
trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl AsAny for i8 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for u8 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for i16 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for u16 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for i32 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for u32 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for i64 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for u64 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for String {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for &'static str {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for NaiveDate {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for NaiveDateTime {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for NaiveTime {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for &dyn QueryParameters<'static> {
    fn as_any(&self) -> &dyn std::any::Any {
        &self as &dyn std::any::Any
    }
}

/// Defines a trait for represent type bounds against the allowed
/// datatypes supported by Canyon to be used as query parameters
pub trait QueryParameters<'a>: Sync + Send {
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync + 'a);
}

impl IntoSql<'_> for &dyn QueryParameters<'_> {
    fn into_sql(self) -> ColumnData<'static> {
        let s = self.clone_from(&self);

        let casted = match (&*self).as_any().type_id() {
            String => match (&*self).as_any().clone().downcast_ref::<String>() {
                Some(v) => ColumnData::String(Some(Cow::from(v.as_str()))),
                None => todo!(),
            },
            i32 => match (&*self).as_any().downcast_ref::<i32>() {
                Some(v) => ColumnData::I32(Some(*v)),
                None => todo!(),
            },
        };

        casted
    }
}

impl<'a> QueryParameters<'a> for i32 {
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync + 'a) {
        match self.as_any().downcast_ref::<i32>() {
            Some(b) => b,
            None => panic!("Bad conversion of parameters"),
        }
    }
}
impl<'a> QueryParameters<'a> for i64 {
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync + 'a) {
        match self.as_any().downcast_ref::<i64>() {
            Some(b) => b,
            None => panic!("Bad conversion of parameters"),
        }
    }
}
impl<'a> QueryParameters<'a> for String {
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync + 'a) {
        match self.as_any().downcast_ref::<String>() {
            Some(b) => b,
            None => panic!("Bad conversion of parameters"),
        }
    }
}
impl<'a> QueryParameters<'a> for &String {
    fn as_postgres_param(&self) -> &(dyn ToSql + Sync + 'a) {
        match self.as_any().downcast_ref::<&str>() {
            Some(b) => b,
            None => panic!("Bad conversion of parameters"),
        }
    }
}
// TODO Scapes lifetimes of 'static on Any
// impl<'a> QueryParameters<'a> for &str {
//     fn as_postgres_param(&self) -> &(dyn ToSql + Sync + 'a) {
//         let a: Box<&dyn AsAny> = Box::new(self);
//         match self.as_any().downcast_ref::<String>() {
//             Some(b) => b,
//             None => panic!("Bad conversion of parameters"),
//         }
//     }
// }

// impl<'a> QueryParameters<'a> for &[u8] {
//     fn as_postgres_param(&self) -> &(dyn ToSql + Sync + 'a) {
//         let a: Box<&dyn AsAny> = Box::new(self);
//         match a.as_any().downcast_ref::<&[u8]>() {
//             Some(b) => b,
//             None => panic!("Bad conversion of parameters"),
//         }
//     }
// }

// impl<'a> QueryParameters<'a> for &'a (dyn ToSql + Sync + Send) {}
// impl<'a> QueryParameters<'a> for &'a dyn IntoSql<'a> {}


/// Defines a trait for make a placeholder when the type it's required
/// by a bound in the callable function, but there's usually an
/// empty `&[]` value, because that query does not need to bound
/// any parameter to the generated query
pub trait PlaceholderType<'a>: QueryParameters<'a> {}
// impl<'a> PlaceholderType<'a> for &'a [u8] {}
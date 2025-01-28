use std::collections::HashMap;

use queries_for_sqlx::SupportNamedBind;
use serde::Serialize;
use serde_json::Value;
use sqlx::{sqlite::SqliteRow, Database, Sqlite};

use crate::{
    
    queries_bridge::{InsertSt, CreatTableSt, SelectSt, UpdateSt},
};

pub trait Validate {
    type Value;
    type Error: Serialize;
    fn validate_on_insert(
        &self,
        value: &mut Self::Value,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn validate_on_update(
        &self,
        value: &mut Self::Value,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub trait DynValidate {
    fn validate_on_insert(
        &self,
        value: &mut Value,
    ) -> Result<(), Value>;
    fn validate_on_update(
        &self,
        value: &mut Value,
    ) -> Result<(), Value>;
}

pub trait Collection<S>: Sized + Send + Sync {
    type PartailCollection;
    fn on_migrate(stmt: &mut CreatTableSt<S>)
    where
        S: Database + SupportNamedBind;
    fn on_update(
        stmt: &mut UpdateSt<S>,
        this: Self::PartailCollection,
    ) -> Result<(), String>
    where
        S: Database + SupportNamedBind;
    fn members() -> &'static [&'static str];
    fn members_scoped() -> &'static [&'static str];

    fn table_name() -> &'static str;
    fn on_select(stmt: &mut SelectSt<S>)
    where
        S: Database + SupportNamedBind;
    fn from_row_noscope(row: &<S as Database>::Row) -> Self
    where
        S: Database;
    fn from_row_scoped(row: &<S as Database>::Row) -> Self
    where
        S: Database;
    fn on_insert(
        self,
        stmt: &mut InsertSt<S>,
    ) -> Result<(), String>
    where
        S: Database + SupportNamedBind;
}

/// used in update operation, similar to Option<T> but implement Serialize and Deserialize
/// differently
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[allow(non_camel_case_types)]
pub enum Update<T> {
    keep,
    set(T),
}

mod impl_deserialize {
    use std::{
        fmt::{self},
        marker::PhantomData,
    };

    use serde::{
        de::{self, Visitor},
        ser::SerializeTupleStruct,
        Deserialize, Deserializer, Serialize, Serializer,
    };

    use super::Update;

    impl<T: Serialize> Serialize for Update<T> {
        fn serialize<S>(
            &self,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut worker = serializer
                .serialize_tuple_struct("optiona_update", 2)?;

            match self {
                Update::keep => {
                    worker.serialize_field("keep")?;
                }
                Update::set(some) => {
                    worker.serialize_field("set")?;
                    worker.serialize_field(some)?;
                }
            };

            worker.end()
        }
    }

    impl<'de, T> Deserialize<'de> for Update<T>
    where
        T: Deserialize<'de>,
    {
        fn deserialize<D>(deser: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deser.deserialize_option(OptionVisitor(PhantomData))
        }
    }

    struct OptionVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for OptionVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Update<T>;

        fn expecting(
            &self,
            formatter: &mut fmt::Formatter,
        ) -> fmt::Result {
            formatter.write_str("update_option")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Update::keep)
        }

        #[inline]
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Update::keep)
        }

        fn visit_some<D>(
            self,
            deser: D,
        ) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            SecondDeser::<Update<T>>::deserialize(deser)
                .map(|e| return e.0)
        }

        fn __private_visit_untagged_option<D>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, ()>
        where
            D: Deserializer<'de>,
        {
            match T::deserialize(deserializer) {
                Ok(ok) => Ok(Update::set(ok)),
                Err(err) => Err(()),
            }
        }
    }

    struct SecondDeser<T>(T);
    struct SecondVisitor<T>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>> Deserialize<'de>
        for SecondDeser<Update<T>>
    {
        fn deserialize<D>(
            deserializer: D,
        ) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer
                .deserialize_tuple_struct(
                    "update_option",
                    2,
                    SecondVisitor(PhantomData),
                )
                .map(|e| SecondDeser(e))
        }
    }

    impl<'de, T: Deserialize<'de>> Visitor<'de>
        for SecondVisitor<T>
    {
        type Value = Update<T>;

        fn expecting(
            &self,
            formatter: &mut fmt::Formatter,
        ) -> fmt::Result {
            formatter.write_str("update_option")
        }
        fn visit_seq<A>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let first = match seq.next_element::<String>()? {
                Some(ok) => ok,
                None => {
                    return Err(de::Error::invalid_length(
                        0,
                        &"len should be exactly 2",
                    ))
                }
            };

            if first == "keep" {
                match seq.size_hint() {
                    Some(0) => {}
                    Some(_) => {
                        return Err(de::Error::custom(
                            "invalid len",
                        ))
                    }
                    None => {}
                }

                return Ok(Update::keep);
            }

            if first == "set" {
                match seq.size_hint() {
                    Some(1) => {}
                    Some(_) => {
                        return Err(de::Error::custom(
                            "invalid len",
                        ))
                    }
                    None => {}
                }

                return Ok(match seq.next_element::<T>()? {
                    Some(ok) => Update::set(ok),
                    None => {
                        return Err(de::Error::invalid_length(
                            1,
                            &"len should be exactly 2",
                        ))
                    }
                });
            }

            return Err(de::Error::custom(
                "has to be either \"set\" or \"keep\"",
            ));
        }
    }
}

pub trait HasCol<C> {
    type This;
    fn name() -> &'static str;
}

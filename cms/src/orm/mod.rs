#![allow(unused)]

use std::{
    collections::HashMap, error::Error, fmt::Write,
    marker::PhantomData,
};

use queries_for_sqlx::{
    ident_safety::PanicOnUnsafe, prelude::stmt,
    quick_query::QuickQuery,
};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use sqlx::{sqlite::SqliteRow, Sqlite};
pub mod dynamic_schema;

pub mod operations;
pub mod queries;
pub mod queries_bridge;
pub mod relations;
pub mod tuple_impls;

pub mod error {
    use axum::{http::StatusCode, response::IntoResponse, Json};
    use serde::Serialize;
    use serde_json::json;

    pub mod insert {
        use axum::{
            http::StatusCode, response::IntoResponse, Json,
        };
        use serde_json::json;

        use crate::orm::error::ErrorInternal;

        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord,
        )]
        pub struct InsertError(StatusCode, InsertErrorInternal);
        impl InsertError {
            pub fn to_refactor(
                code: StatusCode,
                for_dev: &str,
            ) -> Self {
                InsertError(
                    code,
                    InsertErrorInternal::Other(
                        for_dev.to_string(),
                    ),
                )
            }
        }

        impl IntoResponse for InsertError {
            fn into_response(self) -> axum::response::Response {
                let mut body = json!({
                    "status": self.0.as_u16(),
                    "error": self.0.canonical_reason().unwrap_or_default(),
                });

                if let InsertErrorInternal::Other(o) = self.1 {
                    body.as_object_mut()
                        .unwrap()
                        .insert("for_dev".to_string(), o.into());
                }

                (self.0, Json(body)).into_response()
            }
        }

        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord,
        )]
        pub enum InsertErrorInternal {
            ForiegnKeyViolation,
            Other(String),
            Unkown,
        }

        impl From<super::GlobalError> for InsertError {
            fn from(value: super::GlobalError) -> Self {
                let int = match value.1 {
                    ErrorInternal::MoreInfo(info) => {
                        InsertErrorInternal::Other(info)
                    }
                    _ => InsertErrorInternal::Unkown,
                };
                Self(value.0, int)
            }
        }
    }

    #[derive(Debug, Default, Serialize)]
    enum ErrorInternal {
        #[default]
        Unkown,
        EntryNotFound(String),
        MoreInfo(String),
    }

    #[derive(Debug)]
    pub struct GlobalError(StatusCode, ErrorInternal);

    impl From<String> for GlobalError {
        fn from(value: String) -> Self {
            GlobalError(
                StatusCode::BAD_REQUEST,
                ErrorInternal::MoreInfo(value),
            )
        }
    }

    impl GlobalError {
        pub fn info(mut self, msg: &str) -> Self {
            *&mut self.1 =
                ErrorInternal::MoreInfo(msg.to_owned());
            self
        }
    }

    impl IntoResponse for GlobalError {
        fn into_response(self) -> axum::response::Response {
            let mut body = json!({
                "status": self.0.as_u16(),
                "error": self.0.canonical_reason().unwrap_or_default(),
            });

            if let ErrorInternal::MoreInfo(o) = self.1 {
                body.as_object_mut()
                    .unwrap()
                    .insert("info".to_string(), o.into());
            }

            (self.0, Json(body)).into_response()
        }
    }

    pub fn entry_not_found(entry: &str) -> GlobalError {
        GlobalError(
            StatusCode::NOT_FOUND,
            ErrorInternal::EntryNotFound(entry.to_string()),
        )
    }

    pub fn to_refactor(info: &str) -> GlobalError {
        GlobalError(
            StatusCode::BAD_REQUEST,
            ErrorInternal::MoreInfo(info.to_string()),
        )
    }

    pub fn not_found(id: i32) -> GlobalError {
        GlobalError(
            StatusCode::NOT_FOUND,
            ErrorInternal::default(),
        )
    }
    pub fn missing_id_in_query() -> GlobalError {
        GlobalError(
            StatusCode::BAD_REQUEST,
            ErrorInternal::default(),
        )
    }
}

pub trait HasCol<C> {
    type This;
    fn name() -> &'static str;
}

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

pub trait Collection: Sized + Send + Sync {
    type PartailCollection;
    fn on_update1(
        stmt: &mut stmt::UpdateSt<Sqlite, QuickQuery>,
        this: Self::PartailCollection,
    ) -> Result<(), String>;
    fn on_update_ref_mod(
        this: Value,
        stmt: &mut stmt::UpdateSt<Sqlite, QuickQuery>,
        mods: &HashMap<
            String,
            Vec<Box<dyn DynValidate + Send + Sync>>,
        >,
    );

    // done
    fn table_name1() -> &'static str;
    // done
    fn on_select1(
        stmt: &mut stmt::SelectSt<
            Sqlite,
            QuickQuery,
            PanicOnUnsafe,
        >,
    );
    // done
    fn from_row_noscope2(row: &SqliteRow) -> Self;
    fn from_row_scoped2(row: &SqliteRow) -> Self;
    // why I modifing on_get at the first place??
    fn on_get_no_mods(row: &mut SqliteRow) -> Self;
    fn on_insert1(
        self,
        stmt: &mut stmt::InsertStOne<'_, Sqlite>,
    ) -> Result<(), String>;
    fn on_insert_returning() -> Vec<&'static str>;
    fn on_insert_ref_mod(
        this: Value,
        stmt: &mut stmt::InsertStOne<'_, Sqlite>,
        mods: &HashMap<
            String,
            Vec<Box<dyn DynValidate + Send + Sync>>,
        >,
    );

    // why Send+Sync: see comments on TrivialCollection
    fn get_all_modifiers(
    ) -> HashMap<String, Vec<Box<dyn DynValidate + Send + Sync>>>;
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
        fmt::{self, Formatter},
        marker::PhantomData,
    };

    use serde::{
        de::{self, EnumAccess, VariantAccess, Visitor},
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

    // enum Fields {
    //     Keep,
    //     Set,
    // }
    // struct FieldVisitor;
    // impl<'de> de::Visitor<'de> for FieldVisitor {
    //     type Value = Fields;
    //     fn expecting(
    //         &self,
    //         fmtr: &mut Formatter,
    //     ) -> fmt::Result {
    //         // Formatter::write_str(
    //         //     fmtr,
    //         //     "variant identifier",
    //         // )
    //         fmtr.write_str("variant identifier")
    //     }
    //     fn visit_u64<E>(self, val: u64) -> Result<Self::Value, E>
    //     where
    //         E: de::Error,
    //     {
    //         match val {
    //             0u64 => Ok(Fields::Keep),
    //             1u64 => Ok(Fields::Set),
    //             _ => Err(de::Error::invalid_value(
    //                 de::Unexpected::Unsigned(val),
    //                 &"variant index 0 <= i < 2",
    //             )),
    //         }
    //     }
    //     fn visit_str<E>(
    //         self,
    //         val: &str,
    //     ) -> Result<Self::Value, E>
    //     where
    //         E: de::Error,
    //     {
    //         match val {
    //             "keep" => Ok(Fields::Keep),
    //             "set" => Ok(Fields::Set),
    //             _ => Err(de::Error::unknown_variant(
    //                 val, VARIANTS,
    //             )),
    //         }
    //     }
    //     fn visit_bytes<__E>(
    //         self,
    //         val: &[u8],
    //     ) -> Result<Self::Value, __E>
    //     where
    //         __E: de::Error,
    //     {
    //         match val {
    //             b"keep" => Ok(Fields::Keep),
    //             b"set" => Ok(Fields::Set),
    //             _ => {
    //                 let __value = &String::from_utf8_lossy(val);
    //                 Err(de::Error::unknown_variant(
    //                     __value, VARIANTS,
    //                 ))
    //             }
    //         }
    //     }
    // }
    // impl<'de> Deserialize<'de> for Fields {
    //     #[inline]
    //     fn deserialize<D>(deser: D) -> Result<Self, D::Error>
    //     where
    //         D: Deserializer<'de>,
    //     {
    //         Deserializer::deserialize_identifier(
    //             deser,
    //             FieldVisitor,
    //         )
    //     }
    // }
    //
    // struct UpdateVisitor<'de, T>(
    //     PhantomData<(Update<T>, &'de ())>,
    // );
    //
    // impl<'de, T> Visitor<'de> for UpdateVisitor<'de, T>
    // where
    //     T: Deserialize<'de>,
    // {
    //     type Value = Update<T>;
    //     fn expecting(
    //         &self,
    //         fmtr: &mut Formatter,
    //     ) -> fmt::Result {
    //         Formatter::write_str(fmtr, "enum Update")
    //     }
    //     fn visit_enum<A>(
    //         self,
    //         data: A,
    //     ) -> Result<Self::Value, A::Error>
    //     where
    //         A: EnumAccess<'de>,
    //     {
    //         match EnumAccess::variant(data)? {
    //             (Fields::Keep, variant) => {
    //                 de::VariantAccess::unit_variant(variant)?;
    //                 Ok(Update::keep)
    //             }
    //             (Fields::Set, variant) => Result::map(
    //                 de::VariantAccess::newtype_variant::<T>(
    //                     variant,
    //                 ),
    //                 Update::set,
    //             ),
    //         }
    //     }
    // }
    // const VARIANTS: &'static [&'static str] = &["keep", "set"];
}

mod target {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    struct Taget<T>(String, T);
}

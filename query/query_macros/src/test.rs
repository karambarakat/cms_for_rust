use proc_macro2::Span;
use quote::quote;
use syn::{parse_quote, Ident, ItemFn};

#[test]
fn into_query_derive() {
    let input = quote! {
        struct Input<T> {
            id: T,
            name: String,
        }
    };

    let parsed = syn::parse2(input).unwrap();
    let expect = super::into_query::main(parsed);

    let to_be = quote! {
        const _: () = {
            use ::sqlx::Database;
            use ::sqlx::Type;
            use ::sqlx::Encode;
            use ::sqlx::Arguments;
            use ::sqlx::database::HasArguments;
            use ::queries_for_sqlx::*;

            impl<'q, DB, T,>
            ConsumeIntoArguments<'q, DB>
            for Input<T>
            where
                DB: Database,
                for<'r> T: Type<DB> + Encode<'r, DB> + Send + 'q,
                for<'r> String: Type<DB> + Encode<'r, DB> + Send + 'q
            {
                fn add_into_argument(
                    self,
                    argument: &mut <DB as HasArguments<'q> >::Arguments,
                )
                {
                    argument.add(self.id);
                    argument.add(self.name);
                }
            }

            impl<T> TableIdentifiers for Input<T> {
                fn table_name() -> &'static str {
                    stringify!(Input)
                }
            }
            impl<T> ColumnIdentifiers for Input<T> {
                fn columns() -> &'static [&'static str] {
                    &[
                        "id",
                        "name",
                    ]
                }
            }
        };

    };

    assert_eq!(expect.to_string(), to_be.to_string());
}

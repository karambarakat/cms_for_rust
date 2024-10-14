use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemTrait};

use crate::invariant_macro;

fn better_debug(ts: TokenStream, to_be: TokenStream) {
    let mut expect = match parse2::<syn::File>(ts.clone()) {
        Ok(file) => file.items.into_iter(),
        Err(_) => {
            panic!("ts is not valid File: {:?}", ts.to_string())
        }
    };
    let mut to_be = match parse2::<syn::File>(to_be.clone()) {
        Ok(file) => file.items.into_iter(),
        Err(_) => {
            panic!("to_be is not valid File: {:?}", to_be.to_string())
        }
    };

    while to_be.len() != 0 {
        match to_be.next() {
            Some(next) => {
                match expect.next() {
                    Some(expect) => {
                        let expect_ = quote!(#next).to_string();
                        let to_be = quote!(#expect).to_string();
                        if expect_ != to_be {
                            panic!(
                                "expect and to_be are not equal \n{:?} \n{:?}",
                                expect_, to_be
                            )
                        }
                    }
                    None => {
                        panic!(
                            "expecting more item \n{:?}",
                            quote!(#next).to_string()
                        )
                    }
                };
            }
            None => {
                if expect.len() != 0 {
                    panic!(
                        "extra items are not expected \n{:?}",
                        quote!(#(#expect)*).to_string()
                    )
                }
            }
        }
    }
}

#[test]
fn test_2() {
    let input = parse2::<ItemTrait>(quote! {
        pub trait Tuple<'q, G: Inner<'q>> {
            fn part_mut_ctx_mut<E: Example<'q, G>>(part: &mut E, ctx: &mut Mid) {
                if INDEX != 0 {
                    ctx.fn_on_mid_type(", ");
                    part.fm_on_example_trait_1(ctx);
                } else {
                    part.fm_on_example_trait_2(ctx);
                }
            }
            fn my_behavior(&mut self) -> Mid {
                let mut output = Mid::new_on_mid_type();
                self.tuple_mut_ctx_mut(&mut output);
                output
            }
        }
    })
    .unwrap();

    let input = invariant_macro::main(input);

    better_debug(
        input,
        quote! {
            pub struct TupleInvariant<'q, G: Inner<'q>> (
                ::std::marker::PhantomData<(&'q (), G, )>
            );

            impl<'q, G: Inner<'q>, > 
                ::tuples_op::Context<'m', 2> 
                for TupleInvariant <'q, G,>
            {
                type Context = Mid;
            }

            impl<'q, G: Inner<'q>, E: Example<'q, G>> 
            ::tuples_op::Mut<E> for TupleInvariant<'q, G,>
            {
                fn each
                    <const INDEX: usize, const LEN: usize>
                    (part: &mut E, ctx: &mut Self::Context)
                {
                    if INDEX != 0 {
                        ctx.fn_on_mid_type(", ");
                        part.fm_on_example_trait_1(ctx);
                    } else {
                        part.fm_on_example_trait_2(ctx);
                    }
                }
            }

            pub trait Tuple<'q, G: Inner<'q>, > {
                fn tuple_mut_ctx_mut(&mut self, ctx: &mut Mid);
                fn my_behavior(&mut self) -> Mid;
            }

            impl<'q, T, G: Inner<'q>,> Tuple<'q, G,> for T
            where
                T: Sized,
                T: ::tuples_op::behaviors::MutOp<TupleInvariant<'q, G,> >,
            {
                #[inline]
                fn tuple_mut_ctx_mut(&mut self, ctx: &mut Mid) {
                    <T as ::tuples_op::behaviors::MutOp<_> >::each(self, ctx);
                }
                fn my_behavior(&mut self) -> Mid {
                    let mut output = Mid::new_on_mid_type();
                    self.tuple_mut_ctx_mut(&mut output);
                    output
                }
            }
        },
    );
}

use crate::error::exports::*;
use std::ops::Not;
use std::slice::Iter;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::visit::Visit;
use syn::{
    parse_macro_input, parse_quote, FnArg, GenericArgument, Generics,
    ImplItem, ItemImpl, ItemStruct, ItemTrait, TraitItem,
    TraitItemFn,
};
use syn::{GenericParam, Ident};

use crate::error::MacroError;

macro_rules! unwrap {
    ($unw:expr, msg: $msg:literal, into: $errors:expr, span: $expr:expr) => {
        match $unw {
            Some(s) => s,
            None => {
                let span = syn::spanned::Spanned::span(&$expr);
                $errors.extend(quote::quote_spanned! {span=>
                    compile_error!($msg);
                });
                return;
            }
        }
    };
}

macro_rules! better_panic {
    (msg: $msg:literal, into: $errors:expr, span: $expr:expr) => {
        let span = $expr.span();
        $errors.extend(quote_spanned! {span=>
            compile_error!($msg);
        });
        return;
    };
}

macro_rules! unwrap_str_err {
    ($unw:expr, into: $errors:expr, span: $expr:expr) => {
        match &$unw {
            Ok(s) => s,
            Err(e) => {
                let span = syn::spanned::Spanned::span(&$expr);
                if let Err(e) = $unw {
                    $errors.extend(quote::quote_spanned! {span=>
                        compile_error!(#e);
                    });
                }
                return;
            }
        }
    };
}

macro_rules! unwrap_err {
    ($unw:expr, msg: $msg:literal, into: $errors:expr, span: $expr:expr) => {
        match $unw {
            Ok(s) => s,
            Err(e) => {
                let span = $expr.span();
                $errors.extend(quote::quote_spanned! {span=>
                    compile_error!($msg);
                });
                return;
            }
        }
    };
}
struct ParamsGeneris {
    params: Braketed<GenericParam>,
    args: Braketed<GenericArgument>,
}

impl<'a> Visit<'a> for ParamsGeneris {
    fn visit_lifetime_param(&mut self, i: &'a syn::LifetimeParam) {
        let lt_ = &i.lifetime;

        self.params.0.push(parse_quote! { #i });
        self.args.0.push(parse_quote! { #lt_ });
    }
    fn visit_type_param(&mut self, i: &'a syn::TypeParam) {
        let ident = &i.ident;

        self.params.0.push(parse_quote! { #i });
        self.args.0.push(parse_quote! { #ident });
    }
    fn visit_const_param(&mut self, i: &'a syn::ConstParam) {
        let ident = &i.ident;

        self.params.0.push(parse_quote! { #i });
        self.args.0.push(parse_quote! { #ident });
    }
}

struct Errors(TokenStream);

impl Errors {
    fn new() -> Self {
        Self(quote!())
    }
    fn error<T: Spanned>(&mut self, t: T, msg: &str) {
        let span = t.span();
        self.0.extend(quote_spanned!(span=>
            compile_error!(#msg);
        ));
    }
}

struct Immutable {
    invariant_ident: Ident,
    trait_ident: Ident,
    vis: syn::Visibility,
}

struct MutInvariant<'q> {
    errors: &'q mut Errors,
    item: &'q ItemTrait,
    central: &'q Immutable,
    phantom: Vec<TokenStream>,
}

impl<'q> ToTokens for MutInvariant<'q> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let phantom = &self.phantom;
        let gen_param = &self.item.generics;
        let invariant_str = &self.central.invariant_ident;

        tokens.extend(quote! {
            pub struct #invariant_str #gen_param (
                ::std::marker::PhantomData<(#(#phantom,)*)>
            );
        })
    }
}

impl<'q> Visit<'q> for MutInvariant<'q> {
    fn visit_type_param(&mut self, i: &'q syn::TypeParam) {
        let ident = &i.ident;
        self.phantom.push(quote!(#ident));
    }
    fn visit_lifetime_param(&mut self, i: &'q syn::LifetimeParam) {
        let lt_ = &i.lifetime;
        self.phantom.push(quote! { & #lt_ () });
    }
    fn visit_const_param(&mut self, i: &'q syn::ConstParam) {
        let ident = &i.ident;
        self.phantom.push(quote! { #ident });
    }
}

struct Braketed<T>(Vec<T>);

impl<T: ToTokens> Braketed<T> {
    fn no_braket(&self) -> TokenStream {
        let Braketed(inner) = self;
        quote! { #(#inner,)* }
    }
    fn with_braket(&self) -> TokenStream {
        if self.0.is_empty() {
            quote! {}
        } else {
            let Braketed(inner) = self;
            quote! { < #(#inner,)* > }
        }
    }
}

struct MutImpls<'q> {
    errors: &'q mut Errors,
    center: &'q Immutable,
    trait_items: Vec<TraitItem>,
    impl_items: Vec<ImplItem>,
    // params: Braketed<GenericParam>,
    // args: Braketed<GenericArgument>,
    params_generis: ParamsGeneris,
    wheres: Vec<TokenStream>,
    impls: Vec<ItemImpl>,
}

impl<'q> ToTokens for MutImpls<'q> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = &self.center.vis;
        let trait_ident = &self.center.trait_ident;
        let trait_items = &self.trait_items;
        let params = self.params_generis.params.no_braket();
        let params_braket = self.params_generis.params.with_braket();
        let args = self.params_generis.args.with_braket();
        let wheres = &self.wheres;
        let impl_items = &self.impl_items;
        let impls = &self.impls;

        tokens.extend(quote! {
            #(#impls)*
            #vis trait #trait_ident #params_braket {
                #(#trait_items)*
            }
            impl<T, #params> #trait_ident #args for T where T: Sized, #( #wheres, )* {
                #(#impl_items)*
            }
        });
    }
}

mod impl_visitor_for_impls {
    use crate::error::exports::*;
    use crate::error::{unwrap_error, MacroError};

    use super::MutImpls;
    use proc_macro::Ident;
    use proc_macro2::TokenStream;
    use syn::{
        parse_quote,
        spanned::Spanned,
        token::Ge,
        visit::{visit_type, visit_type_reference, Visit},
        FnArg, TraitItemFn,
    };

    struct GetFirstTypeParamIdent {
        ident: Option<syn::Ident>,
        param: Option<syn::TypeParam>,
    }

    impl GetFirstTypeParamIdent {
        fn run(i: &syn::Generics) -> Self {
            let mut g = GetFirstTypeParamIdent {
                ident: None,
                param: None,
            };
            g.visit_generics(i);
            g
        }
    }

    impl<'a> Visit<'a> for GetFirstTypeParamIdent {
        fn visit_type_param(&mut self, i: &'a syn::TypeParam) {
            self.ident = Some(i.ident.clone());
            self.param = Some(i.clone());
        }
    }

    struct GetCtxType<'a> {
        ty: Option<&'a syn::Type>,
        error: &'a mut TokenStream,
        check_for_mutability: bool,
        can_recurse: bool,
    }

    impl<'a> GetCtxType<'a> {
        fn get<'b>(
            check_for_mutablity: bool,
            i: &'b FnArg,
            error: &'b mut TokenStream,
        ) -> Option<&'b syn::Type> {
            let mut result = GetCtxType {
                ty: None,
                error,
                check_for_mutability: check_for_mutablity,
                can_recurse: false,
            };

            result.visit_fn_arg(i);

            result.ty
        }
    }

    impl<'a> Visit<'a> for GetCtxType<'a> {
        fn visit_type_reference(
            &mut self,
            i: &'a syn::TypeReference,
        ) {
            if self.check_for_mutability && i.mutability.is_none() {
                let span = i.span();
                self.error.extend(quote::quote_spanned! {span=>
                    compile_error!("ctx must be a mutable reference");
                });
                return;
            }

            self.can_recurse = true;

            visit_type_reference(self, i);
        }
        fn visit_type(&mut self, i: &'a syn::Type) {
            if self.can_recurse {
                self.ty = Some(i);
            } else {
                visit_type(self, i);
            }
        }
    }

    impl<'a, 'b> Visit<'a> for MutImpls<'b> {
        fn visit_trait_item_fn(&mut self, i: &'a syn::TraitItemFn) {
            match i.sig.ident.to_string().as_str() {
                "last_phantom_ctx_mut" => todo!("part_phantom_ctx_mut_last is implemented in this macro yet"),
                // next
                "last_phantom_ctx_ref" => todo!("part_phantom_ctx_ref_last is not implemented in this macro yet"),
                "part_phantom_ctx_mut" => todo!("part_phantom_ctx_mut is not implemented in this macro yet"),
                "part_phantom_ctx_ref" => todo!("part_phantom_ctx_ref is not implemented in this macro yet"),
                
                "support_fn_once_part_phantom_ctx_ref" => todo!(),
                "support_fn_mut_part_phantom_ctx_ref" => todo!(),
                // next
                "support_fn_part_phantom_ctx_ref" => todo!(),

                "last_consume_ctx_mut" => todo!("part_consume_ctx_mut_last is implemented in this macro yet"),
                "last_consume_ctx_ref" => todo!("part_consume_ctx_ref_last is not implemented in this macro yet"),
                "part_consume_ctx_mut" => todo!("part_consume_ctx_mut is not implemented in this macro yet"),
                "part_consume_ctx_ref" => todo!("part_consume_ctx_ref is not implemented in this macro yet"),

                "last_ref_ctx_mut" => todo!("part_ref_ctx_mut_last is implemented in this macro yet"),
                // next
                "last_ref_ctx_ref" => todo!("part_ref_ctx_ref_last is not implemented in this macro yet"),
                "part_ref_ctx_mut" => todo!("part_ref_ctx_mut is not implemented in this macro yet"),
                "part_ref_ctx_ref" => todo!("part_ref_ctx_ref is not implemented in this macro yet"),

                "last_mut_ctx_mut" => todo!("part_mut_ctx_mut_last is implemented in this macro yet"),
                "last_mut_ctx_ref" => todo!("part_mut_ctx_ref_last is not implemented in this macro yet"),

                // fix
                "part_mut_ctx_mut" => todo!("part_mut_ctx_mut is implemented in this macro yet"),
                "part_mut_ctx_ref" => todo!("part_mut_ctx_ref is not implemented in this macro yet"),

                "each_part" => todo!("each_part is implemented in this macro yet"),
                "each_part_last" => todo!("each_part is implemented in this macro yet"),

                _ => copy_other(self, i),
            }.handle_and_default(&mut self.errors.0)
        }
    }

    fn copy_other<'a, 'b>(
        this: &mut MutImpls<'b>,
        i: &'a TraitItemFn,
    ) 
        -> Result<(), MacroError<TO_BE_HANDLED_LATER>>
    {
        let mut i2 = i.clone();
        i2.default = None;
        this.trait_items.push(parse_quote! {
            #i2
        });

        this.impl_items.push(parse_quote! {
            #i
        });

        Ok(())
    }

    fn impl_part_mut_ctx_mut<'a, 'b>(
        this: &mut MutImpls<'b>,
        i: &'a TraitItemFn,
    ) -> Result<(), MacroError<TO_BE_HANDLED_LATER>> {
        let params = this.params_generis.params.with_braket();
        let args = this.params_generis.args.with_braket();
        let punc = i.sig.inputs.iter().collect::<Vec<_>>();

        use syn::FnArg;
        let ctx = punc.get(1);

        let ctx = unwrap_error!(ctx.to_unwrap(
            "second argument of part_mut_ctx_mut must be ctx",
            Some(i.sig.span())
        ));
        
        let ty = GetCtxType::get(
            true,
            ctx,
            &mut this.errors.0
        ).should_work();

        let invariant_ident = &this.center.invariant_ident;

        let part_ident2 =
            GetFirstTypeParamIdent::run(&i.sig.generics);

        let part_ident = unwrap_error!(
            part_ident2.ident.to_unwrap(
                "part_mut_ctx_mut must have a type parameter",
                Some(i.sig.generics.span())
            )
        );

        let part_param = unwrap_error!(
            part_ident2.param.to_unwrap(
                "part_mut_ctx_mut must have a type parameter",
                Some(i.sig.generics.span())
            )
        );

        this.impls.push(parse_quote! {
            impl #params
            ::tuples_op::Context<'m', 2>
            for #invariant_ident #args
            {
                type Context = #ty;
            }
        });

        let body = &i.default.clone();
        let body = unwrap_error!(
            body.as_ref().to_unwrap(
                "part_mut_ctx_mut must have a default body",
                Some(i.default.span())
            )
        );

        let params_2 = this.params_generis.params.no_braket();

        this.impls.push(parse_quote! {
            impl <#params_2 #part_param>
            ::tuples_op::Mut<#part_ident>
            for #invariant_ident #args
            {
                fn each<const INDEX: usize, const LEN: usize>(
                    part: &mut #part_ident,
                    ctx: &mut Self::Context
                ) #body
            }
        });

        this.trait_items.push(parse_quote! {
            fn tuple_mut_ctx_mut(
                &mut self,
                ctx: &mut #ty
            );
        });

        this.wheres.push(parse_quote! {
            T: ::tuples_op::behaviors::MutOp<#invariant_ident #args>
        });

        this.impl_items.push(parse_quote! {
            #[inline]
            fn tuple_mut_ctx_mut(
                &mut self,
                ctx: &mut #ty
            ) {
                <T as ::tuples_op::behaviors::MutOp<_>>::each(self, ctx);
            }
        });

        Ok(())
    }
}

pub fn main(item: ItemTrait) -> TokenStream {
    let trait_ident = item.ident.clone();
    let invariant_ident = Ident::new(
        &format!("{}Invariant", item.ident.clone()),
        Span::call_site(),
    );
    let vis = item.vis.clone();

    let mut errors = Errors::new();
    let mut output = quote!();

    let center = Immutable {
        invariant_ident,
        trait_ident,
        vis,
    };

    let mut visitor_1 = MutInvariant {
        errors: &mut errors,
        phantom: Default::default(),
        central: &center,
        item: &item,
    };

    visitor_1.visit_generics(&item.generics);

    output.extend(quote! { #visitor_1 });

    let mut params_generis = ParamsGeneris {
        params: Braketed(Default::default()),
        args: Braketed(Default::default()),
    };

    params_generis.visit_generics(&item.generics);

    let mut visitor_2 = MutImpls {
        errors: &mut errors,
        center: &center,
        trait_items: Default::default(),
        impl_items: Default::default(),
        params_generis,
        wheres: Default::default(),
        impls: Default::default(),
    };

    visitor_2.visit_item_trait(&item);

    output.extend(quote! { #visitor_2 });

    let errors = &errors.0;

    output.extend(quote! { #errors });

    output
}


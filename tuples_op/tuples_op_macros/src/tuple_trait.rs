use std::ops::Not;

/* use std::ops::{Deref, DerefMut, Not};
*/
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, visit::Visit, Generics, Ident, ItemTrait, TraitItem, TraitItemFn, Type, TypeParam};

mod invariant {
    use proc_macro2::TokenStream;
    use syn::{visit::Visit, Generics};

    use super::Errors;
    use quote::quote;

    pub struct Invariant<'e> {
        pub errs: &'e mut Errors,
        pub generic: Option<Generics>,
        pub pd: Vec<TokenStream>,
    }
    impl<'a> Visit<'a> for Invariant<'_> {
        fn visit_generics(&mut self, i: &'a syn::Generics) {
            self.generic = Some(i.clone());
            syn::visit::visit_generics(self, i);
        }
        fn visit_trait_item(&mut self, i: &'a syn::TraitItem) {
            // don't go deeper, so we don't visit the generics
            // params of associated items
        }
        fn visit_type_param(&mut self, i: &'a syn::TypeParam) {
            let ident = &i.ident;
            self.pd.push(quote!(#ident));
        }
        fn visit_const_param(&mut self, i: &'a syn::ConstParam) {
            let ident = &i.ident;
            self.pd.push(quote!(#ident));
        }
        fn visit_lifetime_param(&mut self, i: &'a syn::LifetimeParam) {
            let il = &i.lifetime;
            self.pd.push(quote!(& #il ()));
        }
    }

    impl Invariant<'_> {
        pub fn extend(self, ts: &mut TokenStream) {
            let Self { errs, generic, pd } = self;

            ts.extend(quote!(
                pub struct Invariant #generic (
                    ::std::marker::PhantomData<(
                        #(#pd),*
                    )>
                );
            ));
        }
    }
}

mod trait_item_visitor {
    use std::ops::Not;

    use proc_macro2::{Span, TokenStream};
    use quote::quote;
    use syn::{parse_quote, GenericParam, Lifetime, Type, TypeParam};
    use syn::{visit::Visit, Generics, TraitItem};
    use syn::{Ident, TraitItemFn};

    use super::Errors;

    pub struct TraitItemVisitor<'e> {
        pub errs: &'e mut Errors,
        pub trait_generics: Generics,
        pub rest: Rest,
        pub each: Vec<Each>,
        pub pd: Vec<TokenStream>,
    }

    impl TraitItemVisitor<'_> {
        pub fn extend(self, ts: &mut TokenStream) -> Result<(), Errors> {
            for each in self.each {
                if each.errs.0.is_empty().not() {
                    return Err(each.errs);
                }
                each.extend_ts(ts, &self.trait_generics);
            }

            if self.rest.errs.0.is_empty().not() {
                return Err(self.rest.errs);
            }

            self.rest.extend_ts(ts, &self.trait_generics);

            Ok(())
        }
    }

    pub struct Rest {
        pub errs: Errors,
    }

    impl Rest {
        pub fn extend_ts(self, ts: &mut TokenStream, tg: &Generics) {}
    }
    pub enum TyLt {
        Owned,
        Ref,
        Mut,
    }

    #[derive(Default)]
    pub struct Each {
        pub errs: Errors,
        pub e_ident: Option<Ident>,
        pub typ: Option<TypeParam>,
        pub part_ty_lt: Option<TyLt>,
        pub ctx_ty_lt: Option<TyLt>,
        pub output_ty_lt: Option<TyLt>,
    }

    impl Each {
        pub fn extend_ts(self, ts: &mut TokenStream, tg: &Generics) {
            let part_ty_lt = self.part_ty_lt.expect("internal: part_ty_lt");
            let ctx_ty_lt = self.ctx_ty_lt.expect("internal: ctx_ty_lt");

            let mut initials = String::default();

            let mut lts = vec![
                Lifetime {
                    apostrophe: Span::call_site(),
                    ident: Ident::new("tuple_ops_context", Span::call_site()),
                },
                Lifetime {
                    apostrophe: Span::call_site(),
                    ident: Ident::new("tuple_ops_output", Span::call_site()),
                },
            ];

            match &part_ty_lt {
                TyLt::Owned => initials.push('C'),
                TyLt::Ref => {
                    initials.push('R');
                    lts.push(Lifetime {
                        apostrophe: Span::call_site(),
                        ident: Ident::new("tuple_ops", Span::call_site()),
                    })
                }
                TyLt::Mut => {
                    initials.push('M');
                    lts.push(Lifetime {
                        apostrophe: Span::call_site(),
                        ident: Ident::new("tuple_ops", Span::call_site()),
                    })
                }
            }

            match &ctx_ty_lt {
                TyLt::Owned => panic!("internal: ctx cannot be owned"),
                TyLt::Ref => initials.push('R'),
                TyLt::Mut => initials.push('M'),
            }

            let mut ctx_trait = Ident::new(&format!("::tuple_ops::Context{}", &initials), Span::call_site());
            let invaraint_params: Vec<GenericParam> = todo!();
            let each_param = self.typ.expect("internal: typ");
            let each_ident = each_param.ident.clone();
            let invariant_generics: Generics = todo!();
            let context_ty: Type = todo!();

            let part_trait = Ident::new(&format!("Part{}", &initials), Span::call_site());

            ts.extend(quote!(
                impl<
                    #(#lts,)*
                    #(#invaraint_params,)*
                > #ctx_trait <#(#lts),*>
                for Invariant #invariant_generics {
                    type Context = #context_ty;
                }

                impl <
                    #each_param,
                    #(#lts,)*
                    #(#invaraint_params,)*
                >
                #part_trait <#(#lts,)* #each_ident>
                for Invariant #invariant_generics {
                    type Output = #output_ty;
                    fn part<const INDEX: usize, const LEN: usize>(
                        #part_fn_arg,
                        #part_fn_arg,
                    ) -> Self::Output {
                        E::from_row(row)
                    }
                }
            ));
        }
    }

    impl<'a> Visit<'a> for Each {
        fn visit_block(&mut self, node: &'a syn::Block) {
            // don't go deeper
        }

        fn visit_generics(&mut self, n: &'a syn::Generics) {
            let mut iter = n.params.iter();
            if let Some(next) = iter.next() {
                if let syn::GenericParam::Type(ty) = next {
                    self.e_ident = Some(ty.ident.clone());
                    return self.typ = Some(ty.clone());
                }
                self.errs.push("only type params are acceptable", next);
            }
            if let Some(next) = iter.next() {
                self.errs.push("only one type params is acceptable", next);
            }
        }

        fn visit_signature(&mut self, i: &'a syn::Signature) {
            let mut input = i.inputs.iter().cloned();

            self.visit_generics(&i.generics);
            let e_ident = self.e_ident.as_ref().expect("interal: should visit generics before inputs").clone();

            struct EachInfo {
                part_lt: TyLt,
                ctx_lt: TyLt,
                output_lt: TyLt,
            }

            let mut first_arg = FirstArg(None);
            let mut second_arg = SecondArg {
                e_ident,
                error: Default::default(),
                ty_lt: None,
            };

            match input.next() {
                Some(fn_arg) => first_arg.visit_fn_arg(&fn_arg),
                None => return self.errs.push("expected first arg which refer to each element of the tuple", i),
            }
            match input.next() {
                Some(fn_arg) => first_arg.visit_fn_arg(&fn_arg),
                None => return self.errs.push("expected second arg which refer to the context of the operation tuple", i),
            }

            if let Some(next) = input.next() {
                self.errs.push("only two args are allowed", &next);
                return;
            }
        }
    }

    impl<'a> Visit<'a> for Rest {
        fn visit_trait_item_fn(&mut self, i: &'a TraitItemFn) {}
    }

    impl<'a> Visit<'a> for TraitItemVisitor<'_> {
        fn visit_trait_item(&mut self, i: &'a TraitItem) {
            match i {
                TraitItem::Fn(fn_item) => {
                    let fn_name = fn_item.sig.ident.to_string();
                    match fn_name.as_str() {
                        "each" => {
                            let mut each: Each = Default::default();
                            each.visit_trait_item_fn(fn_item);
                            self.errs.0.extend(each.errs.0.drain(..));
                            self.each.push(each);
                        }
                        _ => {
                            self.rest.visit_trait_item_fn(fn_item);
                        }
                    }
                }
                TraitItem::Macro(spanned) => self.errs.push("macro item is not suppored", spanned),
                _ => todo!("trait item not supported"),
            }
        }
    }

    pub struct FirstArg(Option<TyLt>);
    impl<'a> Visit<'a> for FirstArg {}
    pub struct SecondArg {
        pub ty_lt: Option<TyLt>,
        pub error: Errors,
        pub e_ident: Ident,
    }

    impl<'a> Visit<'a> for SecondArg {
        fn visit_type_macro(&mut self, i: &'a syn::TypeMacro) {
            self.error.push("macros are magic!! remove them", i);
        }
        fn visit_ident(&mut self, i: &'a Ident) {
            if i.eq(&self.e_ident) {
                self.error.push("tuple part cannot be mentioned in context", i);
            }
        }
    }
}

pub fn main(input: ItemTrait) -> TokenStream {
    let mut ts = quote!();
    let mut errs = Errors::default();

    let mut invariant = invariant::Invariant {
        errs: &mut errs,
        generic: Default::default(),
        pd: Default::default(),
    };

    invariant.visit_item_trait(&input);
    invariant.extend(&mut ts);

    let mut trait_item_visitor = trait_item_visitor::TraitItemVisitor {
        errs: &mut errs,
        trait_generics: input.generics.clone(),
        rest: trait_item_visitor::Rest { errs: Default::default() },
        each: Default::default(),
        pd: Default::default(),
    };

    trait_item_visitor.visit_item_trait(&input);
    let mut rest_errs = trait_item_visitor.rest.errs.0.clone();
    let mut rest_errs = rest_errs.drain(..);

    let res = trait_item_visitor.extend(&mut ts);
    if let Err(mut err) = res {
        errs.0.extend(err.0.drain(..));
    }
    errs.0.extend(rest_errs);

    if errs.0.is_empty().not() {
        errs.merge(&mut ts);
        return ts;
    }

    ts
}

#[derive(Default)]
struct Errors(Vec<TokenStream>);
impl Errors {
    fn push<S: Spanned>(&mut self, msg: &str, span: &S) {
        use syn::spanned::Spanned;
        let spanned = span.span();
        let msg = msg.to_string();
        self.0.push(quote::quote_spanned!(spanned=> compile_error!(#msg)));
    }
    fn merge(self, sp: &mut TokenStream) {
        for each in self.0 {
            sp.extend(quote::quote!(#each;));
        }
    }
}

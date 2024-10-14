// todo: support &self and &mut self for the extension trait
// todo: support last + the return of last is the return of the
// extension's method
// todo: support init_ctx + the argument of init_ctx is the
// argument of the extension's method + init_ctx should not
// have a return if ctx is not received in the base's for_each
// todo: extend_to_fn
// todo: allow part to be output as well as input
use std::ops::Not;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse2, parse_quote, parse_str, punctuated::Punctuated,
    spanned::Spanned, visit::Visit, FnArg, GenericArgument,
    GenericParam, Ident, ImplItemFn, ItemConst, ItemFn, ItemImpl,
    ItemTrait, PatType, PredicateType, ReturnType, Signature, Stmt,
    Token, TraitItemFn, Type, TypeParam, TypeTuple, WhereClause,
    WherePredicate,
};

enum PartReciever {
    Owned,
    Ref,
    RefMut,
}

enum CtxReciever { 
    Ref,
    RefMut,
}

struct ForEachInfo {
    predicate: PredicateType,
    part_reciever: PartReciever,
    ctx_reciever: CtxReciever,
    ctx_ty: Type,
}

enum Ends {
    OneFnReturn,
    OneFnNoReturn,
    // I made decicion no to handle BothFnNoReturn
    OnlyWithLastReturn,
    BothReturn,
}

pub fn main(item: TokenStream) -> TokenStream {
    let mut stream = quote! {};

    ////////////// parse input as item trait
    let mut base_trait = match parse2::<ItemTrait>(item) {
        Ok(ok) => ok,
        Err(e) => return e.to_compile_error(),
    };

    ////////////// 0. base trait
    base_trait.supertraits.push(parse_quote! { Sized });

    stream.extend(quote! {
        #base_trait
    });

    ////////////// generate identifiers and names
    let base_trait_ident_str = base_trait.ident.clone().to_string();
    if base_trait_ident_str.ends_with("Base").not() {
        return "for explicitness trait name should end with Base"
            .compile_error_spanned(base_trait.ident.span());
    }

    let extention_trait_ident_str = base_trait_ident_str
        .strip_suffix("Base")
        .unwrap()
        .to_string();
    let extention_trait_ident =
        Ident::new(&extention_trait_ident_str, Span::call_site());

    let mut lower_case_fn = String::new();
    let orig = extention_trait_ident.to_string();
    lower_case_fn
        .push(orig.chars().nth(0).unwrap().to_ascii_lowercase());
    for c in extention_trait_ident.to_string()[1..].chars() {
        if c.is_uppercase() {
            lower_case_fn.push('_');
            lower_case_fn.push(c.to_ascii_lowercase());
        } else {
            lower_case_fn.push(c);
        }
    }
    let lower_case_extention =
        Ident::new(&lower_case_fn, Span::call_site());

    let base_trait_ident = base_trait.ident.clone();

    //////////////// 1. impl_base_over_t
    let mut trait_generics = base_trait.generics.clone();
    let mut generic_argument =
        Punctuated::<GenericArgument, Token![,]>::new();

    for each in trait_generics.params.iter() {
        match each {
            syn::GenericParam::Lifetime(lt) => {
                generic_argument.push(parse_quote! {#lt})
            }
            syn::GenericParam::Type(syn::TypeParam {
                ident, ..
            }) => generic_argument.push(parse_quote! {#ident}),
            syn::GenericParam::Const(syn::ConstParam {
                ident,
                ..
            }) => generic_argument.push(parse_quote! {#ident}),
        }
    }
    let len = generic_argument.len();
    let mut generic_argument = quote! { #generic_argument };
    if len != 0 {
        generic_argument = quote! { < #generic_argument > };
    }

    let mut impl_base_over_t: ItemImpl = parse_quote! {
        impl #trait_generics
            #base_trait_ident
            #generic_argument
        for T {}
    };

    impl_base_over_t.generics.params.push(parse_quote! {T});

    stream.extend(quote! {
        #impl_base_over_t
    });

    /////////////// process all trait items
    let mut for_each = None;
    let mut with_last = None;
    let mut extend_to_fn = None;

    for item in &base_trait.items {
        match item {
            syn::TraitItem::Fn(item) => {
                let name = item.sig.ident.to_string();
                let name = name.as_str();
                match name {
                    "for_each" => for_each = Some(item.clone()),
                    "with_last" => with_last = Some(item.clone()),
                    "extend_to_fn" => {
                        extend_to_fn = Some(item.clone())
                    }
                    _ => {
                        return "only for_each, with_last or init_ctx are accepted as item fn in this impl"
                            .compile_error_spanned(item.span());
                    }
                }
            }
            _ => {
                // err!(item, "items other than fn are not supported");
                return "items other than fn are not supported"
                    .compile_error_spanned(item.span());
            }
        }
    }

    let for_each = match for_each {
        Some(ok) => ok,
        None => {
            return "trait should contain for_each method"
                .compile_error()
        }
    };

    let foreach_params = for_each
        .sig
        .generics
        .params
        .clone()
        .into_iter()
        .collect::<Vec<_>>();

    let for_each_params = for_each.sig.generics.params.clone();

    let predicate = match foreach_params.get(0) {
            Some(GenericParam::Type(ty)) => {
                PredicateType {
                    lifetimes: None,
                    bounded_ty: {
                        let ident = ty.ident.clone();
                        parse2::<Type>(quote! {#ident}).unwrap()
                    },
                    colon_token: Default::default(),
                    bounds: ty.bounds.clone(),
                }
            },
            _ => {
                return "for_each should accept at least one parameter like P: Display"
                    .compile_error_spanned(
                        for_each.sig.generics.params.span(),
                    )
            }
        };

    if let Some(index) = foreach_params.get(1) {
        let q = quote!(#index).to_string();
        if q != "INDEX : usize" {
            return "second generic parameter in for_each should be INDEX : usize"
                .compile_error_spanned(index.span());
        }
    }

    if foreach_params.len() > 2 {
        return "for_each should accept at most 2 parameters"
            .compile_error_spanned(
                for_each.sig.generics.params.span(),
            );
    }

    let mut return_ty = None;

    let foreach_args =
        for_each.sig.inputs.clone().into_iter().collect::<Vec<_>>();

    let for_each_first_arg = foreach_args.first().clone();

    let mut part_reciever = match foreach_args.get(0) {
        Some(FnArg::Typed(reciever)) => {
            use syn::PatType;
            let ty = reciever.ty.to_token_stream().to_string();

            match ty.as_str() {
                "P" => PartReciever::Owned,
                "& P" => PartReciever::Ref,
                "& mut P" => PartReciever::RefMut,
                _ => {
                    return "first argument in for_each should be P, &P or &mut P"
                        .compile_error_spanned(reciever.span());
                }
            }
        }
        _ => {
            return "for_each should accept part as argument"
                .compile_error_spanned(for_each.sig.inputs.span())
        }
    };

    let ctx_reciever;
    let ctx_ty: Type;

    match foreach_args.get(1) {
        Some(patt) => {
            let ty = patt.to_token_stream().to_string();
            if ty.starts_with("ctx : & mut") {
                ctx_reciever = CtxReciever::RefMut;
                ctx_ty = parse_str(
                    ty.strip_prefix("ctx : & mut").unwrap(),
                )
                .unwrap();
            } else if ty.starts_with("ctx : &") {
                ctx_reciever = CtxReciever::Ref;
                ctx_ty =
                    parse_str(ty.strip_prefix("ctx : &").unwrap())
                        .unwrap();
            } else {
                return "second argument in for_each should be either `ctx: &mut _` or `ctx: & _` "
                    .compile_error_spanned(patt.span());
            }
        }
        None => {
            return "for_each should accept ctx even if it is &()"
                .compile_error_spanned(for_each.sig.inputs.span());
        }
    }

    let foreach_returns = match &for_each.sig.output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => {
            return_ty = Some(*ty.clone());
            true
        }
    };

    let for_each_info = ForEachInfo {
        predicate,
        part_reciever,
        ctx_reciever,
        ctx_ty,
    };

    let mut end = if return_ty.is_some() {
        Ends::OneFnReturn
    } else {
        Ends::OneFnNoReturn
    };

    if let Some(with_last) = &with_last {
        let params = with_last.sig.generics.params.clone();
        if for_each_params.to_token_stream().to_string()
            != params.to_token_stream().to_string()
        {
            return "with_last params should be same as for_each"
                .compile_error_spanned(
                    with_last.sig.generics.span(),
                );
        }

        let args = &with_last.sig.inputs;

        if args.first().to_token_stream().to_string()
            != for_each_first_arg.to_token_stream().to_string()
        {
            return "first argument of with_last should match for_each's first argument".compile_error_spanned(args.first().span());
        }

        match args.last() {
            Some(FnArg::Typed(PatType { ty, .. })) => {
                if ty.to_token_stream().to_string()
                    != for_each_info
                        .ctx_ty
                        .to_token_stream()
                        .to_string()
                {
                    return "ctx error: unmactch ctx"
                        .compile_error_spanned(args.last().span());
                }
            }
            _ => {
                return "ctx error"
                    .compile_error_spanned(args.last().span());
            }
        }

        if with_last.sig.inputs.len() > 2 {
            return "with_last should accept at most 2 parameters"
                .compile_error_spanned(
                    for_each.sig.generics.params.span(),
                );
        }

        match &with_last.sig.output {
            ReturnType::Default => {
                if return_ty.is_some() {
                    return "should return what for_each returns"
                        .compile_error_spanned(
                            with_last.sig.output.span(),
                        );
                }
            }
            ReturnType::Type(_, ty) => match return_ty {
                Some(should_be_the_same) => {
                    if should_be_the_same
                        .to_token_stream()
                        .to_string()
                        != ty.to_token_stream().to_string()
                    {
                        return "with_last and for_each return type didn't match".compile_error_spanned(ty.span());
                    }
                    end = Ends::BothReturn;
                }
                None => {
                    end = Ends::OnlyWithLastReturn;
                }
            },
        }
    }

    /////////////// 2. extention trait
    let vis = base_trait.vis.clone();

    let main_self_reciever = match &for_each_info.part_reciever {
        PartReciever::Owned => quote!(self),
        PartReciever::Ref => quote!(&self),
        PartReciever::RefMut => quote!(&mut self),
    };

    let init_args = match (&for_each_info.ctx_reciever, 
        
    stream.extend(quote! {
        #vis trait #extention_trait_ident #trait_generics
            : #base_trait_ident <#generic_argument>
        {
            fn #lower_case_extention (#main_self_reciever, #init_args) #main_fn_output;
        }
    });

    for i in 0..=8 {
        let mut type_tuple: TypeTuple = parse_quote! {()};

        let mut body = match &init_ctx.is_some() {
            true => quote! {
                // quote! {
                //     let mut ctx = < #ctx_ty >::default();
                // }
                let params = init_params.clone().into_iter().map(|each| {
                    match each {
                        syn::GenericParam::Type(ty) => {
                            let ident = ty.ident.clone();
                            // quote! {#ident}
                            quote! {}
                        }
                        syn::GenericParam::Lifetime(lt) => {
                            // quote! {#lt}
                            quote! {}
                        }
                        syn::GenericParam::Const(ct) => {
                            let ident = ct.ident.clone();
                            // quote! {#ident}
                            quote! {}
                        }
                    }
                }).collect::<Vec<_>>();

                let params2 = if params.is_empty().not() {

                    // quote! { ::< #(#params),* > }
                    quote!{}
                } else {
                    // quote! {#params}
                    quote!{}
                };

                let args = init_args.clone().into_iter().map(|each| {
                    match each {
                        syn::FnArg::Typed(ty) => {
                            let pat = ty.pat.clone();
                            // quote! {#pat}
                            quote! {}
                        }
                        syn::FnArg::Receiver(_) => {
                            panic!("receiver not allowed in init_ctx")
                        }
                    }
                }).collect::<Vec<_>>();

                // quote!(let mut ctx = Self::init_ctx #params2 ( #(#args,)* ))
                quote! {}
            },

            false => {
                quote! {
                    let mut ctx = < #ctx_ty >::default();
                }
            }
        };

        let mut self_ty = Punctuated::<Type, Token![,]>::new();

        let mut where_clause: Option<WhereClause> = None;
        let mut push_where = |where_pred: WherePredicate| {
            if where_clause.is_none() {
                where_clause = Some(WhereClause {
                    where_token: Token!(where)(Span::call_site()),
                    predicates: Punctuated::new(),
                })
            }
            let mutt = where_clause.as_mut().unwrap();
            mutt.predicates.push(where_pred);
            mutt.predicates.push_punct(parse_quote!(,));
        };

        for j in 0..i {
            let i_ident =
                Ident::new(&format!("P{}", j), Span::call_site());

            let bound = foreach_predicate.bounds.clone();
            push_where(parse_quote!(#i_ident : #bound));

            let index = syn::Index::from(j);

            type_tuple.elems.push(parse_quote!(#i_ident));
            body.extend(quote! {
                <Self as #base_trait_ident <#generic_argument > >::for_each(self.#index, &mut ctx);
            });
            self_ty.push(parse_quote! {#i_ident});
            self_ty.push_punct(Token![,](Span::call_site()));
        }

        // add this only if main returns
        match main_fn_output {
            ReturnType::Default => {}
            ReturnType::Type(_, _) => {
                body.extend(quote! {ctx});
            }
        }

        let mut impl_: ItemImpl = parse_quote! {
            impl #trait_generics #extention_trait_ident <#generic_argument> for ()
                #where_clause
            {
                fn #lower_case_extention #init_params
                ( #main_self_reciever #init_args )
                    #main_fn_output
                {
                    #body

                }
            }
        };

        for j in 0..i {
            let i_ident =
                Ident::new(&format!("P{}", j), Span::call_site());
            impl_.generics.params.push(parse_quote! {#i_ident});
        }

        *impl_.self_ty = parse_quote!( ( #self_ty ));

        stream.extend(quote! {
            #impl_
        });
    }

    stream
}

trait CompileError {
    fn compile_error(self) -> TokenStream;
}

trait CompileErrorSpanned {
    fn compile_error_spanned(self, span: Span) -> TokenStream;
}

impl CompileError for &'static str {
    fn compile_error(self) -> TokenStream {
        quote! {compile_error!(#self)}
    }
}

impl CompileErrorSpanned for &'static str {
    fn compile_error_spanned(self, span: Span) -> TokenStream {
        quote_spanned! {span => compile_error!(#self)}
    }
}

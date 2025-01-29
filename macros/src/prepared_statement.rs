#![allow(unused)]
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::visit::visit_block;
use syn::visit::visit_expr;
use syn::visit::Visit;
use syn::ItemFn;
use syn::Result as SynResult;

pub struct Input {
    fn_item: ItemFn,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(Input {
            fn_item: input.parse()?,
        })
    }
}

pub fn main(input: Input) -> TokenStream {
    struct GetInfo<'a> {
        ident: Option<&'a Ident>,
    }
    struct Validator<'a> {
        first: GetInfo<'a>,
        second: GetInfo<'a>,
    }

    impl<'a> Visit<'a> for GetInfo<'a> {
        fn visit_pat_ident(&mut self, i: &'a syn::PatIdent) {
            self.ident = Some(&i.ident);
        }
    }

    impl<'a> Visit<'a> for Validator<'a> {
        fn visit_signature(&mut self, i: &'a syn::Signature) {
            let arg_len = i.inputs.iter().collect::<Vec<_>>();

            if arg_len.len() != 2 {
                abort!(i.inputs.span(), "input are not 1")
            }

            self.first.visit_fn_arg(&arg_len[0]);
            self.second.visit_fn_arg(&arg_len[1]);
        }
        fn visit_generic_param(
            &mut self,
            i: &'a syn::GenericParam,
        ) {
            abort!(i.span(), "generic is not supported")
        }
    }

    let mut validate = Validator {
        first: GetInfo {
            ident: None,
        },
        second: GetInfo {
            ident: None,
        },
    };

    validate.visit_item_fn(&input.fn_item);

    let first_ident = match validate.first.ident {
        Some(ident) => ident.clone(),
        None => {
            abort!(
                input.fn_item.sig.inputs.span(),
                "no ident for first arg"
            )
        }
    };

    struct ContainIdent<'a>(&'a Ident, bool);

    impl<'a> Visit<'a> for ContainIdent<'a> {
        fn visit_ident(&mut self, i: &'a proc_macro2::Ident) {
            if i == self.0 {
                self.1 = true;
            }
        }
    }

    struct ExpressionVisitor {
        ident: Ident,
        count: usize,
    }

    impl<'a> Visit<'a> for ExpressionVisitor {
        fn visit_expr_if(&mut self, i: &'a syn::ExprIf) {
            let mut any = ContainIdent(&self.ident, false);
            any.visit_expr(&i.cond);
            if any.1 {
                self.count += 1
            }
            visit_block(self, &i.then_branch);
            if let Some((_, exp)) = &i.else_branch {
                visit_expr(self, exp)
            }
        }
    }

    let mut expr = ExpressionVisitor {
        ident: first_ident,
        count: 0,
    };

    expr.visit_block(&input.fn_item.block);

    let item = input.fn_item;
    let mut ts = quote!(#item);

    let count = expr.count;
    ts.extend(quote!(
        mod debugs {
            pub const COUNT: usize = #count;
        }
    ));

    ts
}

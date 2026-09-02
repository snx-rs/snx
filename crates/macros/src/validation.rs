use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Error, Expr, Meta, Result, token::Paren};

pub fn expand_validate(input: DeriveInput) -> Result<TokenStream> {
    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            &input.ident,
            "Validate can only be derived for structs",
        ));
    };

    let mut checks = Vec::new();
    for field in &data.fields {
        let field_ident = field.ident.as_ref().unwrap();
        let mut seen_rules = HashSet::new();

        for attr in field.attrs.iter().filter(|a| a.path().is_ident("rule")) {
            let is_empty = match &attr.meta {
                Meta::List(list) => list.tokens.is_empty(),
                _ => true,
            };

            if is_empty {
                return Err(Error::new_spanned(
                    attr,
                    "expected at least one rule, e.g. #[rule(ascii)]",
                ));
            }

            attr.parse_nested_meta(|meta| {
                let rule_name = meta
                    .path
                    .get_ident()
                    .ok_or_else(|| meta.error("expected a rule name"))?
                    .to_string();

                if !seen_rules.insert(rule_name.clone()) {
                    return Err(meta.error(format!("duplicate rule `{rule_name}` on this field")));
                }

                let variant = format_ident!("{}", to_pascal_case(&rule_name));
                if meta.input.peek(Paren) {
                    let mut seen_args = HashSet::new();
                    let mut fields = Vec::new();

                    meta.parse_nested_meta(|meta| {
                        let key = meta
                            .path
                            .get_ident()
                            .ok_or_else(|| meta.error("expected a field name"))?
                            .clone();

                        if !seen_args.insert(key.to_string()) {
                            return Err(meta.error(format!(
                                "duplicate argument `{key}` for rule `{rule_name}`"
                            )));
                        }

                        let value: Expr = meta.value()?.parse()?;
                        fields.push(quote! { #key: #value });
                        Ok(())
                    })?;

                    checks.push(quote! {
                        <::snx::validation::rules::#variant as ::snx::validation::Rule<_>>::validate(
                            &::snx::validation::rules::#variant { #(#fields),* },
                            &self.#field_ident
                        )?;
                    });
                } else {
                    checks.push(quote! {
                        <::snx::validation::rules::#variant as ::snx::validation::Rule<_>>::validate(
                            &::snx::validation::rules::#variant {},
                            &self.#field_ident
                        )?;
                    });
                }

                Ok(())
            })?;
        }
    }

    let name = input.ident;
    Ok(quote! {
        impl ::snx::validation::Validatable for #name {
            fn validate(self) -> Result<Self, ::snx::validation::Error> {
                #(#checks)*
                Ok(self)
            }
        }
    })
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

use anyhow::Result;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::*;
use quote::quote;
use std::collections::HashMap;
use std::path::Path;

#[proc_macro_error]
#[proc_macro]
pub fn import(_input: TokenStream) -> TokenStream {
    quote! {}.into()
}

fn as_rust_type(ty: &wit_parser::Type) -> syn::Type {
    match ty {
        wit_parser::Type::S64 => syn::parse_quote!(i64),
        wit_parser::Type::U64 => syn::parse_quote!(u64),
        wit_parser::Type::Float64 => syn::parse_quote!(f64),
        wit_parser::Type::Float32 => syn::parse_quote!(f32),
        wit_parser::Type::String => syn::parse_quote!(&str),
        _ => {
            // FIXME
            eprintln!("Unsupported type: {:?}", ty);
            syn::parse_quote!(())
        }
    }
}

fn as_rust_tuple(params: &[(String, wit_parser::Type)]) -> syn::Type {
    let params: Vec<syn::Type> = params.iter().map(|(_, ty)| as_rust_type(ty)).collect();
    syn::parse_quote!((#(#params),*))
}

fn generate_from_wit(wit_path: &Path) -> TokenStream2 {
    let mut tt = Vec::new();
    let interfaces = get_interfaces(wit_path).unwrap();
    for interface in interfaces {
        let module_name = interface.name.as_ref().unwrap();
        let module_ident = syn::Ident::new(module_name, Span::call_site());
        let mut f_tt = Vec::new();
        for (name, f) in interface.functions {
            let ident = syn::Ident::new(&name, Span::call_site());
            let param_names: Vec<_> = f
                .params
                .iter()
                .map(|(name, _)| syn::Ident::new(name, Span::call_site()))
                .collect();
            let param_types: Vec<syn::Type> =
                f.params.iter().map(|(_, ty)| as_rust_type(ty)).collect();
            let input_tt = quote!(#(#param_names: #param_types),*);

            let output = match &f.results {
                wit_parser::Results::Named(params) => as_rust_tuple(params),
                wit_parser::Results::Anon(ty) => as_rust_type(ty),
            };
            f_tt.push(quote! {
                pub fn #ident<'py>(py: Python<'py>, #input_tt) -> PyResult<#output> {
                    todo!()
                }
            });
        }
        tt.push(quote! {
            pub mod #module_ident {
                #(#f_tt)*
            }
        })
    }
    quote! { #(#tt)* }
}

fn get_interfaces(path: &Path) -> Result<Vec<wit_parser::Interface>> {
    let unresolved = wit_parser::UnresolvedPackage::parse_file(path)?;
    let mut wit = wit_parser::Resolve::new();
    wit.push(unresolved, &HashMap::new())?;
    Ok(wit
        .interfaces
        .into_iter()
        .map(|(_id, contents)| contents)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wit_parser() {
        let test_wit = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../test.wit");
        let interfaces = get_interfaces(&test_wit).unwrap();
        dbg!(interfaces);
    }

    fn format(tt: TokenStream2) -> String {
        let input = tt.to_string();
        eprintln!("{}", input);
        prettyplease::unparse(&syn::parse_file(&input).unwrap())
    }

    #[test]
    fn generate_from_test_wit() {
        let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let tt = generate_from_wit(&project_root.join("test.wit"));
        insta::assert_snapshot!(format(tt), @r###"
        pub mod example {
            use pyo3::{prelude::*, types::PyString};
            pub fn a1(py: Python<'_>) -> PyResult<()> {
                let _ = py.import("example")?.getattr("a1")?.call((), None)?;
                Ok(())
            }
            pub fn a2(py: Python<'_>, x: i64) -> PyResult<()> {
                let _ = py.import("example")?.getattr("a2")?.call((x,), None)?;
                Ok(())
            }
            pub fn a3(py: Python<'_>, y: &str, z: f32) -> PyResult<()> {
                let _ = py.import("example")?.getattr("a3")?.call((y, z), None)?;
                Ok(())
            }
            pub fn a4(py: Python<'_>) -> PyResult<i64> {
                let result = py.import("example")?.getattr("a4")?.call((), None)?;
                Ok(result.extract()?)
            }
            pub fn a5(py: Python<'_>, x: i64) -> PyResult<&PyString> {
                let result = py.import("example")?.getattr("a5")?.call((x,), None)?;
                Ok(result.extract()?)
            }
            pub fn a6(py: Python<'_>) -> PyResult<(i64, &PyString)> {
                let result = py.import("example")?.getattr("a6")?.call((), None)?;
                Ok(result.extract()?)
            }
            pub fn a7(py: Python<'_>, x: i64) -> PyResult<(i64, &PyString, f64)> {
                let result = py.import("example")?.getattr("a7")?.call((x,), None)?;
                Ok(result.extract()?)
            }
            pub fn a8<'py>(
                py: Python<'py>,
                x: (i64, &str),
            ) -> PyResult<(i64, &'py PyString, (i64, f64))> {
                let result = py.import("example")?.getattr("a8")?.call((x,), None)?;
                Ok(result.extract()?)
            }
        }
        "###);
    }
}

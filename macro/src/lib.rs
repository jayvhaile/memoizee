use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn memoize(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let fn_name = &sig.ident;
    let inputs = &sig.inputs;
    let block = &input_fn.block;

    let is_async=sig.asyncness.is_some();

    let output_type = match &sig.output {
        ReturnType::Default => {
            return syn::Error::new_spanned(sig, "Functions must have a return type to be memoized.")
                .to_compile_error()
                .into();
        }
        ReturnType::Type(_, ty) => ty.clone(),
    };

    let mut arg_idents = Vec::new();
    let mut arg_types = Vec::new();
    for input in inputs.iter() {
        if let syn::FnArg::Typed(pat_ty) = input {
            if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                arg_idents.push(pat_ident.ident.clone());
                arg_types.push(pat_ty.ty.clone());
            } else {
                return syn::Error::new_spanned(&pat_ty.pat, "Argument must be a simple identifier")
                    .to_compile_error()
                    .into();
            }
        } else {
            return syn::Error::new_spanned(input, "Methods with `self` are not supported by this macro.")
                .to_compile_error()
                .into();
        }
    }

    if arg_idents.len() != 1 {
        return syn::Error::new_spanned(
            sig,
            "The #[memoize] macro currently supports exactly one argument."
        )
            .to_compile_error()
            .into();
    }

    let arg_ident = &arg_idents[0];
    let arg_type = &arg_types[0];

    let fn_name_caps=fn_name.to_string().to_uppercase();
    let memoizer_name = syn::Ident::new(&format!("__{}_MEMOIZER", fn_name_caps), fn_name.span());

    let memoizer_type = if is_async {
        quote! { memoizee::AsyncMemoizer::<#arg_type, #output_type> }
    } else {
        quote! { memoizee::SyncMemoizer::<#arg_type, #output_type> }
    };

    let gen_memoizer = if is_async {
        quote! {
            once_cell::sync::Lazy::new(|| {
                #memoizer_type::new(move |key: #arg_type| {
                    Box::pin(async move {
                        let #arg_ident = key;
                        #block
                    })
                })
            })
        }
    } else {
        quote! {
            once_cell::sync::Lazy::new(|| {
                #memoizer_type::new(move |key: #arg_type| {
                    let #arg_ident = key;
                    #block
                })
            })
        }
    };

    let call_memoizer = if is_async {
        quote! {
            #memoizer_name.of(#arg_ident).await
        }
    } else {
        quote! {
            #memoizer_name.of(#arg_ident)
        }
    };

    let expanded = quote! {
        static #memoizer_name: ::once_cell::sync::Lazy<#memoizer_type> = #gen_memoizer;

        #vis #sig {
            #call_memoizer
        }
    };

    expanded.into()
}
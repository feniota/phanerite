use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, parse_quote};

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let mut generics = input.generics;
    let (_, type_generics, _) = generics.split_for_impl();
    let event_type = quote!(#name #type_generics);
    generics.make_where_clause().predicates.push(parse_quote!(
        #event_type: ::core::marker::Send + ::core::marker::Sync + 'static
    ));
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics ::phanerite_executor::event::Event
            for #name #type_generics #where_clause {}
    }
    .into()
}

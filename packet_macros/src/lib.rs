use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{self, Parser},
    parse_macro_input,
    punctuated::Punctuated,
    Attribute, DeriveInput, Expr, ItemStruct, Lit, Meta, MetaNameValue,
};

// using proc_macro_attribute to declare an attribute like procedural macro
#[proc_macro_derive(Packet, attributes(server_packet))]
// _metadata is argument provided to macro call and _input is code to which attribute like macro attaches
pub fn derive_packet(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let mut serialize_impl: Vec<TokenStream> = Vec::new();
    let mut deserialize_impl: Vec<TokenStream> = Vec::new();

    let ident = ast.ident;

    let packet_id = 0x01;

    quote! {
        impl crate::packet::Packet for #ident {
            const PACKET_ID: i32 = #packet_id;
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn server_packet(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);

    let args =
        parse_macro_input!(args with Punctuated::<MetaNameValue, syn::Token![,]>::parse_terminated);

    let mut packet_id = None;

    for arg in args.iter() {
        if arg.path.is_ident("id") {
            match &arg.value {
                Expr::Lit(lit) => match &lit.lit {
                    Lit::Int(int) => {
                        packet_id = Some(int.base10_parse::<i32>().unwrap());
                    }
                    _ => panic!("Invalid id type"),
                },
                _ => panic!("Invalid id type"),
            }
        }
    }

    let packet_id = packet_id.expect("id attribute is required for server_packet");

    let mut serialize_impl: Vec<proc_macro2::TokenStream> = Vec::new();

    for field in item_struct.fields.iter() {
        let field_ident = field.ident.as_ref().unwrap();

        serialize_impl.push(quote! {
            packet.write(self.#field_ident);
        });
    }

    let ident = &item_struct.ident;

    quote! {
        #[derive(Debug)]
        #item_struct

        impl crate::packet::Packet for #ident {
            const PACKET_ID: i32 = #packet_id;
        }

        impl crate::packet::ServerPacket for #ident {
            fn serialize(self) -> std::io::Result<crate::packet::RawPacket> {
                let mut packet = crate::packet::RawPacket::new(#packet_id);

                #(#serialize_impl)*

                std::io::Result::Ok(packet)
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn client_packet(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);

    let args =
        parse_macro_input!(args with Punctuated::<MetaNameValue, syn::Token![,]>::parse_terminated);

    let mut packet_id = None;

    for arg in args.iter() {
        if arg.path.is_ident("id") {
            match &arg.value {
                Expr::Lit(lit) => match &lit.lit {
                    Lit::Int(int) => {
                        packet_id = Some(int.base10_parse::<i32>().unwrap());
                    }
                    _ => panic!("Invalid id type"),
                },
                _ => panic!("Invalid id type"),
            }
        }
    }

    let packet_id = packet_id.expect("id attribute is required for client_packet");

    let mut deserialize_fields: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut deserialize_struct: Vec<proc_macro2::TokenStream> = Vec::new();

    for field in item_struct.fields.iter() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        deserialize_fields.push(quote! {
            let #field_ident = #field_type::read(&mut bytes)?;
        });
        deserialize_struct.push(quote! {
            #field_ident,
        });
    }

    let ident = &item_struct.ident;

    quote! {
        #[derive(Debug)]
        #item_struct

        impl crate::packet::Packet for #ident {
            const PACKET_ID: i32 = #packet_id;
        }

        impl crate::packet::ClientPacket for #ident {
            fn deserialize(raw_packet: crate::packet::RawPacket) -> std::io::Result<Self> {
                assert_eq!(Self::PACKET_ID, raw_packet.packet_id, "Invalid packet id");

                let mut bytes = raw_packet.cursor();

                #(#deserialize_fields)*

                std::io::Result::Ok(Self {
                    #(#deserialize_struct)*
                })
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn varint(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(args as parse::Nothing);

    let ident = &item_struct.ident;

    return quote! {
        #item_struct
    }
    .into();
}

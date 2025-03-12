mod attrs;

use std::fmt::Display;

use attrs::{parse_field_attrs, PacketOpts};
use convert_case::Casing;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{self, spanned::Spanned, Field};

extern crate proc_macro;

#[proc_macro_derive(Packet, attributes(packet_id, packet_state, packet_direction, packet))]
pub fn packet_derive(input: TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).expect("Failed to parse input");

    let packet = impl_packet_derive(&ast).unwrap_or_else(|err| err);
    let reader = impl_packet_readable_derive(&ast).unwrap_or_else(|err| err);
    let writer = impl_packet_writable_derive(&ast).unwrap_or_else(|err| err);

    quote! {
        #packet
        #reader
        #writer
    }
    .into()
}

#[proc_macro_derive(PacketReadable, attributes(enum_type, packet))]
pub fn packet_readable_derive(input: TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).expect("Failed to parse input");

    impl_packet_readable_derive(&ast)
        .unwrap_or_else(|err| err)
        .into()
}

#[proc_macro_derive(PacketWritable, attributes(packet))]
pub fn packet_writable_derive(input: TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).expect("Failed to parse input");

    impl_packet_writable_derive(&ast)
        .unwrap_or_else(|err| err)
        .into()
}

fn spanned_error<T: ToTokens, D: Display>(
    tokens: T,
    message: D,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    Err(syn::Error::new_spanned(tokens, message)
        .to_compile_error()
        .into())
}

// (ident, reader, writer)
fn field_parser(
    field: &Field,
) -> Result<
    (
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
    ),
    proc_macro2::TokenStream,
> {
    fn spanned_error<T: ToTokens, D: Display>(
        tokens: T,
        message: D,
    ) -> Result<
        (
            proc_macro2::TokenStream,
            proc_macro2::TokenStream,
            proc_macro2::TokenStream,
        ),
        proc_macro2::TokenStream,
    > {
        Err(syn::Error::new_spanned(tokens, message)
            .to_compile_error()
            .into())
    }

    let ident = field.ident.as_ref().unwrap();
    let ty = &field.ty;

    let opts = parse_field_attrs(field)?;

    match ty {
        syn::Type::Path(ty) => {
            let ident = ident.to_token_stream();
            if let Some(repr) = opts.repr {
                let ident_raw = syn::Ident::new(&format!("{}_raw", ident), ident.span());
                let reader = quote! {
                    let #ident_raw: #repr = packet::read::PacketReadable::read(&mut raw)?;
                    let #ident = #ident_raw.into();
                };
                let writer = quote! {
                    let #ident_raw: #repr = self.#ident.into();
                    packet::write::PacketWritable::write(&#ident_raw, &mut raw)?;
                };

                return Ok((ident, reader, writer));
            } else {
                let reader = quote! {
                    let #ident: #ty = packet::read::PacketReadable::read(&mut raw)?;
                };
                let writer = quote! {
                    packet::write::PacketWritable::write(&self.#ident, &mut raw)?;
                };

                return Ok((ident, reader, writer));
            }
        }
        _ => spanned_error(ty, format!("unsupported field type {:?}", ty)),
    }
}

type FieldParserResult = Result<
    (
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
    ),
    proc_macro2::TokenStream,
>;

fn extract_fields(ast: &syn::DeriveInput) -> FieldParserResult {
    fn spanned_error<T: ToTokens, D: Display>(tokens: T, message: D) -> FieldParserResult {
        Err(syn::Error::new_spanned(tokens, message)
            .to_compile_error()
            .into())
    }

    if let syn::Data::Struct(data) = &ast.data {
        if let syn::Fields::Named(fields) = &data.fields {
            let fields = fields.named.iter().map(field_parser).collect::<Vec<_>>();
            let mut idents = Vec::new();
            let mut readers = Vec::new();
            let mut writers = Vec::new();

            for field in fields {
                let (ident, reader, writer) = field?;
                idents.push(ident);
                readers.push(reader);
                writers.push(writer);
            }

            Ok((idents, readers, writers))
        } else {
            spanned_error(ast, "expected named fields")
        }
    } else {
        spanned_error(ast, "expected struct")
    }
}

fn impl_packet_derive(
    ast: &syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    // Get the name of the type for which this macro is applied.
    let name = &ast.ident;

    let packet_id = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("packet_id") {
            attr.parse_args::<syn::LitInt>().ok()
        } else {
            None
        }
    });
    let packet_id = match packet_id {
        Some(id) => id,
        _ => {
            return spanned_error(ast, "packet_id attribute is required");
        }
    };

    let packet_state = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("packet_state") {
            attr.parse_args::<syn::Variant>().ok()
        } else {
            None
        }
    });
    let packet_state = match packet_state {
        Some(state) => state.ident,
        _ => {
            return spanned_error(ast, "packet_state attribute is required");
        }
    };

    let packet_direction = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("packet_direction") {
            attr.parse_args::<syn::Variant>().ok()
        } else {
            None
        }
    });
    let packet_direction = match packet_direction {
        Some(direction) => direction.ident,
        _ => {
            return spanned_error(ast, "packet_direction attribute is required");
        }
    };

    let upper_snake = name.to_string().to_case(convert_case::Case::Constant);
    let descriptor_ident = syn::Ident::new(&format!("_{}_DESCRIPTOR", upper_snake), name.span());

    let gen = quote! {
        #[linkme::distributed_slice(packet::registry::PACKET_REGISTRY)]
        static #descriptor_ident: packet::registry::PacketDescriptor = packet::registry::PacketDescriptor {
            id: #packet_id,
            state: packet::ConnectionState::#packet_state,
            name: stringify!(#name),
            direction: packet::PacketDirection::#packet_direction,
        };

        impl packet::Packet for #name {

            const PACKET_ID: i32 = #packet_id;
            const PACKET_STATE: packet::ConnectionState = packet::ConnectionState::#packet_state;
            const PACKET_DIRECTION: packet::PacketDirection = packet::PacketDirection::#packet_direction;

            // Read and validate packet id, then read as regular struct
            fn read_packet<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                let packet: packet::raw::RawPacket = packet::read::PacketReadable::read(reader)?;
                let mut raw = std::io::BufReader::new(&packet.bytes[..]);

                if Self::PACKET_ID != packet.packet_id {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid packet id",
                    ));
                }

                // Read self as regular struct
                let parsed = packet::read::PacketReadable::read(&mut raw)?;

                Ok(parsed)
            }

            // Extra wrapper around the struct, to include writing length and packet id
            fn write_packet<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                let mut buf = Vec::new();
                {
                    // Final bytes without length varint
                    let mut raw = std::io::BufWriter::new(&mut buf);

                    // Write packet id
                    let packet_id = packet::types::VarInt(Self::PACKET_ID);
                    packet::write::PacketWritable::write(&packet_id, &mut raw)?;
                    // Write self as a struct
                    packet::write::PacketWritable::write(&self, &mut raw)?;
                }

                let len = packet::types::VarInt(buf.len() as i32);

                packet::write::PacketWritable::write(&len, writer)?;
                writer.write_all(&buf)?;

                Ok(())
            }
        }
    };

    Ok(gen.into())
}

fn impl_packet_readable_derive(
    ast: &syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    // Get the name of the type for which this macro is applied.
    let name = &ast.ident;

    let opts = attrs::parse_packet_attrs(ast)?;

    match opts {
        PacketOpts::Struct { repr, .. } => {
            let (idents, readers, _) = extract_fields(ast)?;

            match repr {
                attrs::StructRepr::Flat => {
                    let gen = quote! {
                        impl packet::read::PacketReadable for #name {
                            fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                                let mut raw = reader;

                                #(
                                    #readers
                                )*

                                Ok(Self {
                                    #(
                                        #idents,
                                    )*
                                })
                            }
                        }
                    };

                    return Ok(gen.into());
                }
                attrs::StructRepr::Json => {
                    let gen = quote! {
                        impl packet::read::PacketReadable for #name {
                            fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                                let string: String = packet::read::PacketReadable::read(reader)?;
                                let parsed = serde_json::from_str(&string)?;
                                Ok(parsed)
                            }
                        }
                    };

                    return Ok(gen.into());
                }
            }
        }
        PacketOpts::Enum { repr, data } => {
            let mut variants = Vec::new();
            data.variants.iter().for_each(|v| {
                let ident = &v.ident;
                let value = match &v.discriminant {
                    Some((_, expr)) => expr,
                    None => panic!("Expected enum variant to have a value"),
                };

                variants.push(quote! {
                    #value => Self::#ident,
                });
            });

            let gen = quote! {
                impl packet::read::PacketReadable for #name {
                    fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                        let value: #repr = packet::read::PacketReadable::read(reader)?;

                        let variant = match value.into() {
                            #(
                                #variants
                            )*
                            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid enum value")),
                        };

                        Ok(variant)
                    }
                }
            };

            return Ok(gen.into());
        }
    }
}

fn impl_packet_writable_derive(
    ast: &syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    // Get the name of the type for which this macro is applied.
    let name = &ast.ident;

    let opts = attrs::parse_packet_attrs(ast)?;

    match &opts {
        PacketOpts::Struct { .. } => {}
        PacketOpts::Enum { .. } => {}
    }

    match &opts {
        PacketOpts::Struct { repr, .. } => {
            let (_, _, writers) = extract_fields(ast)?;

            match repr {
                attrs::StructRepr::Flat => {
                    let gen = quote! {
                        impl packet::write::PacketWritable for #name {
                            fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                                let mut raw = writer;

                                #(
                                    #writers
                                )*

                                Ok(())
                            }
                        }
                    };

                    return Ok(gen.into());
                }
                attrs::StructRepr::Json => {
                    let gen = quote! {
                        impl packet::write::PacketWritable for #name {
                            fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                                let string = serde_json::to_string(self)?;
                                packet::write::PacketWritable::write(&string, writer)?;
                                Ok(())
                            }
                        }
                    };

                    return Ok(gen.into());
                }
            }
        }
        PacketOpts::Enum { repr, data } => {
            let mut variants = Vec::new();
            data.variants.iter().for_each(|v| {
                let ident = &v.ident;
                let value = match &v.discriminant {
                    Some((_, expr)) => expr,
                    None => panic!("Expected enum variant to have a value"),
                };

                variants.push(quote! {
                    #name::#ident => #repr::from(#value),
                });
            });

            let gen = quote! {
                impl Into<#repr> for &#name {
                    fn into(self) -> #repr {
                        match self {
                            #(
                                #variants
                            )*
                        }
                    }
                }

                impl packet::write::PacketWritable for #name {
                    fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                        let value: #repr = self.into();
                        packet::write::PacketWritable::write(&value, writer)?;
                        Ok(())
                    }
                }
            };

            return Ok(gen.into());
        }
    }
}

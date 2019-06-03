extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenTree, Punct, Group};
use proc_quote::{quote, quote_spanned,TokenStreamExt};

use syn::parse::{Parse, ParseStream};
use syn::punctuated::{Punctuated, Pair};
use syn::ext::IdentExt;
use syn::{braced, parenthesized, parse_macro_input, token, Ident, Type, Result, Token};

use permutate::{Permutator};

/// Argument typing shorthands:
/// - x: x register
/// - y: y register
/// - r: x or y register
/// TODO: float register, and fl+r reg
/// - c: constant
/// - s: source (register or constant)
/// - l: label
/// - f: failure label? zero meaning raise
/// - a: atom
/// - i: int
/// - u: unsigned int
/// - s: small? arity? (u8) --> could do these as digits 1,2,4,8 (bytes)
/// TODO: t, e

#[derive(Debug)]
struct Arg {
    name: Ident,
    types: Ident,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let types = input.parse()?;
        Ok(Self { name, types })
    }
}

#[derive(Debug)]
struct Instruction {
    fn_token: Token![fn],
    name: Ident,
    args: Punctuated<Arg, Token![,]>,
    // brace: token::Brace,
    body: proc_macro2::TokenStream,
}

impl Parse for Instruction {
    fn parse(input: ParseStream) -> Result<Self> {
        // input.parse::<syn::Token![fn]>()?;
        let fn_token: Token![fn] = input.parse()?;
        let name: Ident = input.parse::<Ident>()?.unraw();
        let head;
        let _paren_token = parenthesized!(head in input);
        let args = head.parse_terminated(Arg::parse)?;
        let content;
        let _brace_token = braced!(content in input);
        let body = content.parse()?;
        Ok(Self {
            fn_token,
            name,
            args,
            body,
        })
    }
}

#[derive(Debug)]
struct Instructions {
    ins: Punctuated<Instruction, Token![,]>,
}

impl Parse for Instructions {
    fn parse(input: ParseStream) -> Result<Self> {
        let ins = input.parse_terminated(Instruction::parse)?;
        Ok(Self {
            ins
        })
    }
}

/// a specialized instruction.
struct Opcode {
    name: Ident,
    genop: Ident,
    args: Vec<(Ident, String)>,
    body: proc_macro2::TokenStream,
}

fn build_variants(ins: &Instruction) -> Vec<Opcode> {
    use heck::CamelCase;
    let name = ins.name.to_string().to_camel_case();
    let genop = Ident::new(&name, Span::call_site());

    if ins.args.is_empty() {
        vec![Opcode {
            name: genop.clone(),
            genop,
            args: vec![],
            body: ins.body.clone(),
        }]
    } else {
	let types: Vec<Vec<String>> = ins.args.iter().map(|arg| arg.types.to_string().chars().map(|c| c.to_string()).collect()).collect();

        // *sigh*

        // Convert the `Vec<Vec<String>>` into a `Vec<Vec<&str>>`
        let tmp: Vec<Vec<&str>> = types.iter()
            .map(|list| list.iter()
                 .map(AsRef::as_ref)
                 .collect::<Vec<&str>>()
            )
            .collect();

        // Convert the `Vec<Vec<&str>>` into a `Vec<&[&str]>`
        let vector_of_slices: Vec<&[&str]> = tmp.iter()
            .map(AsRef::as_ref).collect();

        let permutator = Permutator::new(&vector_of_slices);

        let names = ins.args.iter().map(|arg| arg.name.clone());

        permutator.map(|types| {
            let mut suffix = types.join("");
            suffix.make_ascii_uppercase();
            let name = Ident::new(&format!("{}{}", name, suffix), Span::call_site());

            Opcode {
                name,
                genop: genop.clone(),
                args: names.clone().zip(types.iter().map(|t| t.to_string())).collect(),
                body: ins.body.clone()
            }
        }).collect()
    }
}

fn expand_enum_variants(op: &Opcode) -> proc_macro2::TokenStream {
    let name = &op.name;
    if op.args.is_empty() {
        quote! { #name }
    } else {
        let args: Vec<proc_macro2::TokenStream> = op.args.iter().map(|(arg, t)| {
            let t = match t.as_str() {
                "c" => Ident::new("char", Span::call_site()),
                "x" => Ident::new("u8", Span::call_site()),
                "y" => Ident::new("u16", Span::call_site()),
                _ => panic!()
            };

            quote!{ #arg: #t }
        }).collect();

        quote! {
            #name { #(#args),* }
        }
    }
}

fn expand_impls(op: &Opcode) -> proc_macro2::TokenStream {
    let variant = &op.name;
    let args: Vec<_> = op.args.iter().map(|(n, _t)| n).collect();

    let mut input = op.body.clone().into_iter().peekable();
    let body =  interpolate_opcode_impls(&mut input).unwrap();

    quote! {
        Instruction::#variant { #(#args),* } => {
            #body
        }
    }
}

#[proc_macro]
pub fn ins(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Instructions);

    eprintln!("SYN: {:#?}", input);

    let opcodes: Vec<Opcode> = input.ins.iter().map(build_variants).flatten().collect();

    let enums: Vec<_> = opcodes.iter().map(expand_enum_variants).collect();
    let impls: Vec<_> = opcodes.iter().map(expand_impls).collect();

    let tokens = quote! {
        enum Instruction {
            #(#enums),*
        }

        #[inline(always)]
        fn run(ins: &Instruction) {
           match *ins {
               #(#impls),*
           } 
        }
    };

    tokens.into()
}

// interpolation

/// Alias for common iterator passed to the parsing functions.
type InputIter = std::iter::Peekable<proc_macro2::token_stream::IntoIter>;

/// Returns the interpolation pattern type based on the content of the given
/// `punct` and the rest of the `input`.
///
/// Input that is part of the pattern is automatically consumed.
fn interpolation_pattern_type(
    punct: &Punct,
    input: &mut InputIter,
) -> Option<Ident> {
    match (punct.as_char(), input.peek()) {
        // #ident
        ('#', Some(TokenTree::Ident(_))) => {
            if let Some(TokenTree::Ident(ident)) = input.next() {
                Some(ident)
            } else {
                panic!("guaranteed by previous match")
            }
        }

        // Not an interpolation pattern
        _ => None,
    }
}

/// Transforms a `Group` into code that appends the given `Group` into `__stream`.
///
/// Inside iterator patterns, use `parse_group_in_iterator_pattern`.
fn interpolate_group(stream: &mut proc_macro2::TokenStream, group: &Group) -> Result<()> {
    let mut inner = group.stream().into_iter().peekable();
    let inner = interpolate_opcode_impls(&mut inner)?;

    let mut new = Group::new(group.delimiter(), inner);
    new.set_span(group.span());
    stream.append(new);
    Ok(())
}


/// Interpolates the given variable, which should implement `ToTokens`.
fn interpolate_to_tokens_ident(stream: &mut proc_macro2::TokenStream, ident: &Ident) {
    // let ref_mut_stream = quote! { &mut __stream };
    // let span = ident.span();
    // stream.append_all(quote_spanned! { span=>
    //     ::proc_quote::__rt::append_to_tokens(#ref_mut_stream, & #ident);
    // });
    stream.append_all(quote! { 1 })
}

/// Parses the input according to `quote!` rules.
fn interpolate_opcode_impls(input: &mut InputIter) -> Result<proc_macro2::TokenStream> {
    let mut output = proc_macro2::TokenStream::new();

    while let Some(token) = input.next() {
        match &token {
            TokenTree::Group(group) => interpolate_group(&mut output, group)?,
            TokenTree::Punct(punct) => match interpolation_pattern_type(&punct, input) {
                Some(ident) => {
                    interpolate_to_tokens_ident(&mut output, &ident);
                }
                None => {
                    // pass through
                    output.append(token);
                }
            },
            _ => {
                // pass through
                output.append(token);
            }
        }
    }

    Ok(output)
}

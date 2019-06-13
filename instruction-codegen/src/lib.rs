extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Group, Punct, Span, TokenTree};
use proc_quote::{quote, quote_spanned, TokenStreamExt};

use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, parse_macro_input, Ident, Result, Token};

use permutate::Permutator;

// TODO: skip underscores

/// Argument typing shorthands:
/// - x: x register
/// - y: y register
/// - r: register size
/// - d: x or y register destination)
/// TODO: float register, and fl+r reg
/// - c: constant
/// - s: source (register or constant)
/// - l: label
/// - f: failure label? zero meaning raise
/// - a: atom
/// - i: int
/// - t: tiny unsigned int (u8)
/// - u: unsigned int
/// - s: small? arity? (u8) --> could do these as digits 1,2,4,8 (bytes)
/// - b: bif
/// - m: extended list
/// - v: byte vec (Vec<u8>)
/// - q: extended literal
/// - S: size
/// - F: flag
/// - R: float register or x or y
/// - L: float register
/// - T: jump table

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
        Ok(Self { ins })
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
        // TODO: skip types starting with _
        let types: Vec<Vec<String>> = ins
            .args
            .iter()
            .map(|arg| {
                arg.types
                    .to_string()
                    .chars()
                    .map(|c| c.to_string())
                    .collect()
            })
            .collect();

        // *sigh*

        // Convert the `Vec<Vec<String>>` into a `Vec<Vec<&str>>`
        let tmp: Vec<Vec<&str>> = types
            .iter()
            .map(|list| list.iter().map(AsRef::as_ref).collect::<Vec<&str>>())
            .collect();

        // Convert the `Vec<Vec<&str>>` into a `Vec<&[&str]>`
        let vector_of_slices: Vec<&[&str]> = tmp.iter().map(AsRef::as_ref).collect();

        let permutator = Permutator::new(&vector_of_slices);

        let names = ins.args.iter().map(|arg| arg.name.clone());

        permutator
            .map(|types| {
                let suffix = types.join("");
                // suffix.make_ascii_uppercase();
                let name = Ident::new(&format!("{}_{}", name, suffix), Span::call_site());

                Opcode {
                    name,
                    genop: genop.clone(),
                    args: names
                        .clone()
                        .zip(types.iter().map(|t| t.to_string()))
                        .collect(),
                    body: ins.body.clone(),
                }
            })
            .collect()
    }
}

fn expand_enum_variants(op: &Opcode) -> proc_macro2::TokenStream {
    let name = &op.name;
    if op.args.is_empty() {
        quote! { #name }
    } else {
        let args: Vec<proc_macro2::TokenStream> = op
            .args
            .iter()
            .filter_map(|(arg, t)| {
                if !arg.to_string().starts_with('_') {
                    let t = match t.as_str() {
                        "c" => Ident::new("u32", Span::call_site()),
                        "r" => Ident::new("Regs", Span::call_site()),
                        "x" => Ident::new("RegisterX", Span::call_site()),
                        "y" => Ident::new("RegisterY", Span::call_site()),
                        "i" => Ident::new("i32", Span::call_site()),
                        "q" => Ident::new("u32", Span::call_site()),
                        "l" => Ident::new("Label", Span::call_site()),
                        "s" => Ident::new("Source", Span::call_site()),
                        "t" => Ident::new("u8", Span::call_site()),
                        "u" => Ident::new("u32", Span::call_site()),
                        "d" => Ident::new("Register", Span::call_site()),
                        "b" => Ident::new("Bif", Span::call_site()),
                        "m" => Ident::new("ExtendedList", Span::call_site()),
                        "v" => Ident::new("Bytes", Span::call_site()),
                        "F" => Ident::new("BitFlag", Span::call_site()),
                        "S" => Ident::new("Size", Span::call_site()),
                        "L" => Ident::new("FloatRegs", Span::call_site()),
                        "R" => Ident::new("FRegister", Span::call_site()),
                        "T" => Ident::new("JumpTable", Span::call_site()),
                        //_ => syn::Error::new(arg.span(), format!("unexpected type `{}`", t)).to_compile_error(),
                        _ => panic!("unexpected type {}", t),
                    };

                    Some(quote! { #arg: #t })
                } else {
                    None
                }
            })
            .collect();

        quote! {
            #name { #(#args),* }
        }
    }
}

fn expand_impls(op: &Opcode) -> proc_macro2::TokenStream {
    let variant = &op.name;
    let args: Vec<_> = op
        .args
        .iter()
        .filter_map(|(n, t)| {
            if !n.to_string().starts_with('_') {
                let var = match t.as_str() {
                    "m" => quote! { ref #n },
                    "v" => quote! { ref #n },
                    "T" => quote! { ref #n },
                    _ => quote! { #n },
                };
                Some(var)
            } else {
                // skip types starting with _
                None
            }
        })
        .collect();

    let mut input = op.body.clone().into_iter().peekable();
    let body = interpolate_opcode_impls(op, &mut input).unwrap();

    quote! {
        Instruction::#variant { #(#args),* } => {
            #body
        }
    }
}

fn expand_loads(op: &Opcode) -> proc_macro2::TokenStream {
    let variant = &op.name;
    let genop = &op.genop;
    let matches: Vec<_> = op
        .args
        .iter()
        .map(|(n, t)| {
            if n.to_string().starts_with('_') {
                // blank match on ignored args
                quote! { _ }
            } else {
                match t.as_str() {
                    "c" => quote! { #n }, // could be BigInt, hence why we don't match
                    "x" => quote! { V::X(#n) },
                    "y" => quote! { V::Y(#n) },
                    "i" => quote! { V::Constant(#n) }, // a bit annoying, was cast to constant before
                    "q" => quote! { V::ExtendedLiteral(#n) },
                    "b" => quote! { V::Bif(#n) },
                    "l" => quote! { V::Label(#n) },
                    "L" => quote! { V::FloatReg(#n) },
                    "T" => quote! { V::JumpTable(#n) },
                    "u" => quote! { #n },
                    "t" => quote! { #n },
                    "r" => quote! { #n },
                    _ => quote! { #n },
                }
            }
        })
        .collect();
    let args: Vec<_> = op
        .args
        .iter()
        .filter_map(|(n, t)| {
            if !n.to_string().starts_with('_') {
                let arg = match t.as_str() {
                    "c" => quote_spanned! {n.span() => #n: to_const(#n, constants, literal_heap) },
                    "x" => quote_spanned! {n.span() => #n: RegisterX((*#n).try_into().unwrap()) },
                    "y" => quote_spanned! {n.span() => #n: RegisterY((*#n).try_into().unwrap()) },
                    "i" => quote_spanned! {n.span() => #n: #n.to_int().unwrap() },
                    "q" => quote_spanned! {n.span() => #n: *#n },
                    "b" => quote_spanned! {n.span() => #n: Bif(*#n) },
                    "l" => quote_spanned! {n.span() => #n: *#n },
                    "L" => quote_spanned! {n.span() => #n: (*#n).try_into().unwrap() },
                    "T" => quote_spanned! {n.span() => #n: #n.clone() },
                    "u" => quote_spanned! {n.span() => #n: #n.to_u32().try_into().unwrap() },
                    "t" => quote_spanned! {n.span() => #n: #n.to_u32().try_into().unwrap() },
                    "r" => quote_spanned! {n.span() => #n: #n.to_u32().try_into().unwrap() },
                    _ => {
                        quote_spanned! {n.span() => #n: #n.into_with_heap(constants, literal_heap) }
                    }
                };
                Some(arg)
            } else {
                // skip types starting with _
                None
            }
        })
        .collect();

    // let cond = quote!{ if use_jump_table(&*list) };

    quote! {
        (O::#genop, &[#(#matches),*]) => {
            // let (table, min) = gen_jump_table(&*list, *fail);
            res.push(I::#variant {
                #(#args),*
                // arg: arg.to_val(constants, literal_heap),
                // fail: *fail,
                // table,
                // min
            });
        }
    }
}

#[proc_macro]
pub fn instruction(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Instructions);

    // eprintln!("SYN: {:#?}", input);

    let opcodes: Vec<Opcode> = input.ins.iter().map(build_variants).flatten().collect();

    let enums: Vec<_> = opcodes.iter().map(expand_enum_variants).collect();
    let impls: Vec<_> = opcodes.iter().map(expand_impls).collect();
    let loads: Vec<_> = opcodes.iter().map(expand_loads).collect();

    let tokens = quote! {
        #[derive(Clone, Debug)]
        pub enum Instruction {
            #(#enums),*
        }

        pub trait Captures<'a> {}

        impl<'a, T> Captures<'a> for T {}

        #[inline(always)]
        pub fn run<'a: 'c, 'b: 'c, 'c>(
            vm: &'a Machine,
            process: &'b mut RcProcess,
        ) -> impl std::future::Future<Output = Result<process::State, Exception>> + Captures<'a> + Captures<'b> + 'c {
            async move {  // workaround for https://github.com/rust-lang/rust/issues/56238
            let context = process.context_mut();
            context.reds = 2000; // self.config.reductions;

            // process the incoming signal queue
            process.process_incoming()?;

            // a fake constant nil
            let NIL = Term::nil();
            let mut ins;

            loop {
                ins = unsafe { (*context.ip.module).instructions.get_unchecked(context.ip.ptr as usize) };
                context.ip.ptr += 1;

                // if process.pid >= 95 {
                // info!(
                // "proc pid={:?} reds={:?} mod={:?} offs={:?} ins={:?}",
                // process.pid,
                // context.reds,
                // atom::to_str(unsafe { (*context.ip.module).name}).unwrap(),
                // context.ip.ptr,
                // ins,
                // );
              // }
                match *ins {
                    #(#impls),*
                }
            }
            }
        }

        pub fn transform_engine(
            instrs: &[crate::loader::Instruction],
            constants: &mut Vec<Term>,
            literal_heap: &Heap,
        ) -> Vec<instruction::Instruction> {
            let mut res = Vec::with_capacity(instrs.len());
            let mut iter = instrs.iter();

            use instruction::Instruction as I;
            use crate::opcodes::Opcode as O;
            use LValue as V;

            // transform!(
            //     CallExtLast(u, Bif=b, D) -> Deallocate { n: D } | CallBifOnly { bif: b }
            // )
            while let Some(ins) = iter.next() {
                match (ins.op, &ins.args.as_slice()) {
                    #(#loads),*
                    _ => unimplemented!("{:?}({:?})", ins.op, &ins.args.as_slice())
                }
            }
            res
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
fn interpolation_pattern_type(punct: &Punct, input: &mut InputIter) -> Option<Ident> {
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
fn interpolate_group(
    stream: &mut proc_macro2::TokenStream,
    opcode: &Opcode,
    group: &Group,
) -> Result<()> {
    let mut inner = group.stream().into_iter().peekable();
    let inner = interpolate_opcode_impls(opcode, &mut inner)?;

    let mut new = Group::new(group.delimiter(), inner);
    new.set_span(group.span());
    stream.append(new);
    Ok(())
}

/// Interpolates the given variable, which should implement `ToTokens`.
fn interpolate_to_tokens_ident(
    stream: &mut proc_macro2::TokenStream,
    opcode: &Opcode,
    ident: &Ident,
    next: Option<&proc_macro2::TokenTree>,
) {
    let typ = match opcode.args.iter().find(|(name, _)| name == ident) {
        Some(t) => &t.1,
        None => {
            stream.append_all(
                syn::Error::new_spanned(ident, "argument undefined").to_compile_error(),
            );
            return;
        }
    };

    let code = match next.and_then(|token| match token {
        TokenTree::Punct(punct) => Some(punct.as_char()),
        _ => None,
    }) {
        // if it's an assignment, treat it specially
        Some('=') => match typ.as_str() {
            "c" => syn::Error::new_spanned(ident, "cannot assign to constant").to_compile_error(),
            "q" => syn::Error::new_spanned(ident, "cannot assign to literal").to_compile_error(),
            "x" => {
                quote_spanned!(ident.span() => *unsafe { context.x.get_unchecked_mut(#ident.0 as usize) })
            }
            "y" => quote_spanned!(ident.span() => *unsafe {
                let len = context.stack.len();
                context.stack.get_unchecked_mut(len - (#ident.0 + 1) as usize)
            }),
            "d" => quote_spanned! {ident.span()=>
                let reg = match register {
                    Register::X(reg) => unsafe {
                        self.x.get_unchecked_mut(reg.0 as usize)
                    }
                    Register::Y(reg) => {
                        let len = self.stack.len();
                        &mut self.stack[(len - (reg.0 + 1) as usize)]
                    }
                };
                *reg = value
            },
            "r" => quote_spanned!(ident.span() => #ident),
            "l" => quote_spanned!(ident.span() => #ident),
            "t" => quote_spanned!(ident.span() => #ident),
            "u" => quote_spanned!(ident.span() => #ident),
            "b" => quote_spanned!(ident.span() => #ident),
            _ => syn::Error::new_spanned(ident, format!("unexpected type `{}`", typ))
                .to_compile_error(),
        },
        _ => match typ.as_str() {
            "c" => {
                quote_spanned!(ident.span() => unsafe { (*context.ip.module).constants[#ident as usize] })
            }
            "q" => {
                quote_spanned!(ident.span() => unsafe { (*context.ip.module).literals[#ident as usize] })
            }
            // TODO: assignment
            "x" => {
                quote_spanned!(ident.span() => unsafe { (*context.x.get_unchecked(#ident.0 as usize)) })
            }
            "y" => {
                quote_spanned!(ident.span() => unsafe { (*context.stack.get_unchecked(context.stack.len() - (#ident.0 + 1) as usize)) })
            }
            "s" => quote_spanned!(ident.span() => context.expand_arg(#ident)),
            // TODO: needs to be assignable
            "d" => quote_spanned!(ident.span() => context.fetch_register(#ident)),
            "r" => quote_spanned!(ident.span() => #ident),
            "l" => quote_spanned!(ident.span() => #ident),
            "t" => quote_spanned!(ident.span() => #ident),
            "u" => quote_spanned!(ident.span() => #ident),
            "b" => quote_spanned!(ident.span() => #ident),
            _ => syn::Error::new_spanned(ident, format!("unexpected type `{}`", typ))
                .to_compile_error(),
        },
    };
    stream.append_all(code)
}

/// Parses the input according to `quote!` rules.
fn interpolate_opcode_impls(
    op: &Opcode,
    input: &mut InputIter,
) -> Result<proc_macro2::TokenStream> {
    let mut output = proc_macro2::TokenStream::new();

    while let Some(token) = input.next() {
        match &token {
            TokenTree::Group(group) => interpolate_group(&mut output, op, group)?,
            TokenTree::Punct(punct) => match interpolation_pattern_type(&punct, input) {
                Some(ident) => {
                    interpolate_to_tokens_ident(&mut output, op, &ident, input.peek());
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

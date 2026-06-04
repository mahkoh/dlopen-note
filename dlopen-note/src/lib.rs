//! This crate provides the [`dlopen_note`] proc macro that can be used to attach dlopen
//! metadata to an ELF binary.
//!
//! > Using dlopen() to load optional dependencies brings several advantages:
//! > programs can gracefully downgrade a feature when a library is not available,
//! > and the shared library is only loaded into the process (and its ELF
//! > constructors are run) only when the requested feature is actually used. But
//! > it also has some drawbacks, and the main one is that it is harder to track a
//! > program’s dependencies, since unlike build-time dynamic linking there will
//! > not be a mention in the ELF metadata. This specification aims to solve this
//! > problem by providing a standardized specification for a custom ELF note that
//! > can be used to list dlopen() dependencies.
//!
//! See <https://uapi-group.org/specifications/specs/elf_dlopen_metadata>.

use {
    proc_macro2::{Literal, Span},
    quote::quote,
    syn::{
        Error, Ident, LitStr, Token, bracketed,
        parse::{Parse, ParseStream},
        parse_macro_input,
    },
};

/// Creates a `.note.dlopen` section.
///
/// # Example
///
/// ```rust
/// # use dlopen_note::dlopen_note;
/// dlopen_note! {
///     soname: ["libvulkan.so.1"],
///     feature: "vulkan",
///     description: "required for the vulkan renderer",
///     priority: "recommended",
/// }
/// ```
///
/// # Syntax
///
/// The fields and their meaning are specified at [uapi-group.org].
///
/// In short:
///
/// - The supported fields are the ones that appear in the example above.
/// - The `soname` field is required and must have at least one element.
/// - The strings must not contain any control characters.
/// - The `priority` must be `required`, `recommended`, or `suggested`.
///
/// [uapi-group.org]: https://uapi-group.org/specifications/specs/elf_dlopen_metadata
#[proc_macro]
pub fn dlopen_note(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: Input = parse_macro_input!(input as Input);
    let mut json = vec![];
    json.extend_from_slice(br#"[{"soname":["#);
    for (idx, name) in input.soname.iter().enumerate() {
        if idx > 0 {
            json.push(b',');
        }
        json.push(b'"');
        json.extend_from_slice(&name.value);
        json.push(b'"');
    }
    json.push(b']');
    for (name, field) in [
        ("feature", &input.feature),
        ("description", &input.description),
        ("priority", &input.priority),
    ] {
        if let Some(v) = field {
            json.extend_from_slice(br#",""#);
            json.extend_from_slice(name.as_ref());
            json.extend_from_slice(br#"":""#);
            json.extend_from_slice(&v.value);
            json.extend_from_slice(br#"""#);
        }
    }
    json.extend_from_slice(br#"}]"#);
    json.push(0);
    let len = Literal::usize_unsuffixed(json.len());
    while json.len() % 4 != 0 {
        json.push(0);
    }
    let size = Literal::usize_unsuffixed(json.len());
    let json = Literal::byte_string(&json);
    let out = quote! {
        const _: () = {
            #[repr(C)]
            struct Note {
                namesz: u32,
                descsz: u32,
                r#type: u32,
                name: [u8; 4],
                desc: [u8; #size],
            }
            #[used]
            #[unsafe(link_section = ".note.dlopen")]
            static DLOPEN_NOTE: Note = Note {
                namesz: 4,
                descsz: #len,
                r#type: 0x407c0c0a,
                name: *b"FDO\0",
                desc: *#json,
            };
        };
    };
    out.into()
}

#[derive(Debug)]
struct Input {
    soname: Vec<JsonString>,
    feature: Option<JsonString>,
    description: Option<JsonString>,
    priority: Option<JsonString>,
}

#[derive(Debug)]
struct JsonString {
    span: Span,
    value: Vec<u8>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut soname = None;
        let mut feature = None;
        let mut description = None;
        let mut priority = None;
        fn set<T>(key: Ident, option: &mut Option<T>, value: T) -> syn::Result<()> {
            if option.replace(value).is_some() {
                return Err(Error::new(key.span(), "duplicate key"));
            }
            Ok(())
        }
        loop {
            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            if ident == "soname" {
                let content;
                bracketed!(content in input);
                let vec: Vec<_> = content
                    .parse_terminated(JsonString::parse, Token![,])?
                    .into_iter()
                    .collect();
                if vec.is_empty() {
                    return Err(Error::new(ident.span(), "at least one soname is required"));
                }
                set(ident, &mut soname, vec)?;
            } else if ident == "feature" {
                set(ident, &mut feature, input.parse()?)?;
            } else if ident == "description" {
                set(ident, &mut description, input.parse()?)?;
            } else if ident == "priority" {
                set(ident, &mut priority, input.parse()?)?;
            } else {
                return Err(Error::new(ident.span(), "unknown key"));
            }
            if input.parse::<Option<Token![,]>>()?.is_none() {
                break;
            }
            if input.is_empty() {
                break;
            }
        }
        if !input.is_empty() {
            return Err(input.error("expected a comma"));
        }
        let Some(soname) = soname else {
            return Err(input.error("missing soname field"));
        };
        let input = Self {
            soname,
            feature,
            description,
            priority,
        };
        if let Some(v) = &input.priority
            && !matches!(&*v.value, b"required" | b"recommended" | b"suggested")
        {
            return Err(Error::new(v.span, "unknown priority"));
        }
        Ok(input)
    }
}

impl Parse for JsonString {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit: LitStr = input.parse()?;
        let raw = lit.value();
        let span = lit.span();
        let mut cooked = String::with_capacity(raw.len());
        for c in raw.chars() {
            if c.is_control() {
                return Err(Error::new(
                    span,
                    "strings cannot contain control characters",
                ));
            }
            if c == '\\' || c == '"' {
                cooked.push('\\');
            }
            cooked.push(c);
        }
        Ok(JsonString {
            span,
            value: cooked.into_bytes(),
        })
    }
}

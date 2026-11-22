//! Zero-dependency procedural derive macros for `fy-refine`.
//!
//! Provides four zero-boilerplate `#[derive(...)]` macros for structs and enums:
//! - [`SafeValue`]: emits compile-time exhaustively destructured `SafeValue` implementations;
//! - [`SafeDrop`]: emits compile-time drop-safety verification and `SafeDrop` implementations;
//! - [`SafeClone`]: emits structural field-by-field `Clone` and `SafeClone` implementations;
//! - [`Projectors`]: generates zero-overhead `SafeFunction` field/variant projector types under a matching `snake_case` submodule.

extern crate proc_macro;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

/// Derives the `SafeValue` safety contract for a struct or enum.
///
/// Supports `#[refine(const)]` to generate `const unsafe impl` on nightly,
/// and `#[refine(crate = path)]` to override the root crate path (default `::fy_refine`).
#[proc_macro_derive(SafeValue, attributes(refine))]
pub fn derive_safe_value(input: TokenStream) -> TokenStream {
    expand_contract_derive(input, ContractKind::Value)
}

/// Derives the `SafeDrop` safety contract for a struct or enum.
///
/// Statically verifies at compile time that all fields implement `SafeDrop` and that the target type has no custom `impl Drop`.
#[proc_macro_derive(SafeDrop, attributes(refine))]
pub fn derive_safe_drop(input: TokenStream) -> TokenStream {
    expand_contract_derive(input, ContractKind::Drop)
}

/// Derives a structural `Clone` implementation and `SafeClone` safety contract for a struct or enum.
#[proc_macro_derive(SafeClone, attributes(refine))]
pub fn derive_safe_clone(input: TokenStream) -> TokenStream {
    expand_contract_derive(input, ContractKind::Clone)
}

/// Generates zero-overhead field or variant projectors under a matching `snake_case` submodule.
///
/// - By default projects shared references (`&'a FieldTy`);
/// - Annotate a field or variant with `#[projector(copy)]` to project by value (`FieldTy`);
/// - Annotate a field or variant with `#[projector(skip)]` to skip generating its projector;
/// - Annotate the type with `#[refine(mod = custom_mod)]` to override the submodule name, or `#[refine(const)]` for `const impl`.
#[proc_macro_derive(Projectors, attributes(refine, projector))]
pub fn derive_projectors(input: TokenStream) -> TokenStream {
    expand_projectors(input)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ContractKind {
    Value,
    Drop,
    Clone,
}

struct RefineConfig {
    is_const: bool,
    crate_path: Vec<TokenTree>,
    mod_name: Option<Ident>,
}

impl Default for RefineConfig {
    fn default() -> Self {
        let default_crate: Vec<TokenTree> = "::fy_refine"
            .parse::<TokenStream>()
            .expect("built-in crate path must parse")
            .into_iter()
            .collect();
        Self {
            is_const: false,
            crate_path: default_crate,
            mod_name: None,
        }
    }
}

#[derive(Default, Clone, Copy)]
struct ProjectorAttr {
    is_copy: Option<bool>,
    skip: bool,
}

struct NamedField {
    name: Ident,
    ty: Vec<TokenTree>,
    attr: ProjectorAttr,
}

struct TupleField {
    ty: Vec<TokenTree>,
    attr: ProjectorAttr,
}

enum VariantKind {
    Unit,
    Tuple(Vec<TupleField>),
    Named(Vec<NamedField>),
}

struct VariantDef {
    name: Ident,
    attr: ProjectorAttr,
    kind: VariantKind,
}

enum ItemKind {
    StructUnit,
    StructTuple(Vec<TupleField>),
    StructNamed(Vec<NamedField>),
    Enum(Vec<VariantDef>),
}

struct ParsedItem {
    vis: Vec<TokenTree>,
    name: Ident,
    config: RefineConfig,
    kind: ItemKind,
}

struct ParseError {
    span: Span,
    message: String,
}

impl ParseError {
    fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }

    fn into_compile_error(self) -> TokenStream {
        let mut out = TokenStream::new();
        out.extend([
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("core", self.span)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("compile_error", self.span)),
            TokenTree::Punct(Punct::new('!', Spacing::Alone)),
        ]);
        let mut msg = Literal::string(&self.message);
        msg.set_span(self.span);
        let mut inner = TokenStream::new();
        inner.extend([TokenTree::Literal(msg)]);
        let mut group = Group::new(Delimiter::Parenthesis, inner);
        group.set_span(self.span);
        out.extend([
            TokenTree::Group(group),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);
        out
    }
}

struct TokenWriter {
    stream: TokenStream,
}

impl TokenWriter {
    fn new() -> Self {
        Self {
            stream: TokenStream::new(),
        }
    }

    fn push_str(&mut self, code: &str) {
        self.stream.extend(
            code.parse::<TokenStream>()
                .expect("macro-generated code snippet must lex"),
        );
    }

    fn push_slice(&mut self, tts: &[TokenTree]) {
        self.stream.extend(tts.iter().cloned());
    }

    fn push_ident(&mut self, ident: &Ident) {
        self.stream.extend([TokenTree::Ident(ident.clone())]);
    }

    fn push_index(&mut self, idx: usize, span: Span) {
        let mut lit = Literal::usize_unsuffixed(idx);
        lit.set_span(span);
        self.stream.extend([TokenTree::Literal(lit)]);
    }

    fn push_grouped(&mut self, delim: Delimiter, build: impl FnOnce(&mut TokenWriter)) {
        let mut inner = TokenWriter::new();
        build(&mut inner);
        self.stream
            .extend([TokenTree::Group(Group::new(delim, inner.stream))]);
    }

    fn finish(self) -> TokenStream {
        self.stream
    }
}

fn expand_contract_derive(input: TokenStream, contract: ContractKind) -> TokenStream {
    let parsed = match parse_item(input) {
        Ok(item) => item,
        Err(err) => return err.into_compile_error(),
    };

    let macro_name = match contract {
        ContractKind::Value => "impl_safe_value",
        ContractKind::Drop => "impl_safe_drop",
        ContractKind::Clone => "impl_safe_clone",
    };
    let with_const = contract == ContractKind::Value && parsed.config.is_const;

    let mut w = TokenWriter::new();
    w.push_slice(&parsed.config.crate_path);
    w.push_str("::");
    w.push_str(macro_name);
    w.push_str("!");
    w.push_grouped(Delimiter::Parenthesis, |inner| {
        if with_const {
            inner.push_str("const ");
        }
        match &parsed.kind {
            ItemKind::StructUnit => {
                inner.push_ident(&parsed.name);
            }
            ItemKind::StructNamed(fields) => {
                inner.push_ident(&parsed.name);
                inner.push_grouped(Delimiter::Brace, |body| {
                    for (i, field) in fields.iter().enumerate() {
                        if i > 0 {
                            body.push_str(", ");
                        }
                        body.push_ident(&field.name);
                    }
                });
            }
            ItemKind::StructTuple(fields) => {
                inner.push_ident(&parsed.name);
                inner.push_grouped(Delimiter::Parenthesis, |body| {
                    for i in 0..fields.len() {
                        if i > 0 {
                            body.push_str(", ");
                        }
                        body.push_str(&format!("__fy_f{i}"));
                    }
                });
            }
            ItemKind::Enum(variants) => {
                inner.push_str("enum ");
                inner.push_ident(&parsed.name);
                inner.push_grouped(Delimiter::Brace, |body| {
                    for (vi, variant) in variants.iter().enumerate() {
                        if vi > 0 {
                            body.push_str(", ");
                        }
                        body.push_ident(&variant.name);
                        match &variant.kind {
                            VariantKind::Unit => {}
                            VariantKind::Named(fields) => {
                                body.push_grouped(Delimiter::Brace, |vbody| {
                                    for (fi, field) in fields.iter().enumerate() {
                                        if fi > 0 {
                                            vbody.push_str(", ");
                                        }
                                        vbody.push_ident(&field.name);
                                    }
                                });
                            }
                            VariantKind::Tuple(fields) => {
                                body.push_grouped(Delimiter::Parenthesis, |vbody| {
                                    for fi in 0..fields.len() {
                                        if fi > 0 {
                                            vbody.push_str(", ");
                                        }
                                        vbody.push_str(&format!("__fy_f{fi}"));
                                    }
                                });
                            }
                        }
                    }
                });
            }
        }
    });
    w.push_str(";");
    w.finish()
}

fn expand_projectors(input: TokenStream) -> TokenStream {
    let parsed = match parse_item(input) {
        Ok(item) => item,
        Err(err) => return err.into_compile_error(),
    };

    let mod_name = parsed.config.mod_name.clone().unwrap_or_else(|| {
        let snake = to_snake_case(&parsed.name.to_string());
        let raw_or_ident = escape_keyword(&snake);
        if let Some(stripped) = raw_or_ident.strip_prefix("r#") {
            Ident::new_raw(stripped, parsed.name.span())
        } else {
            Ident::new(&raw_or_ident, parsed.name.span())
        }
    });

    let mut w = TokenWriter::new();
    match &parsed.kind {
        ItemKind::StructUnit => {
            emit_projector_mod(&mut w, &parsed.vis, &mod_name, &[]);
        }
        ItemKind::StructNamed(fields) => {
            let mut proj_names = Vec::new();
            let mut active = Vec::new();
            for f in fields {
                if f.attr.skip {
                    continue;
                }
                let pascal = to_pascal_case(&f.name.to_string());
                let p_ident = Ident::new(&pascal, f.name.span());
                proj_names.push(p_ident.clone());
                active.push((p_ident, f));
            }
            emit_projector_mod(&mut w, &parsed.vis, &mod_name, &proj_names);
            for (proj_ident, field) in active {
                emit_named_struct_projector(
                    &mut w,
                    &parsed.config,
                    &parsed.name,
                    &mod_name,
                    &proj_ident,
                    field,
                );
            }
        }
        ItemKind::StructTuple(fields) => {
            let mut proj_names = Vec::new();
            let mut active = Vec::new();
            for (idx, f) in fields.iter().enumerate() {
                if f.attr.skip {
                    continue;
                }
                let p_ident = Ident::new(&format!("F{idx}"), parsed.name.span());
                proj_names.push(p_ident.clone());
                active.push((idx, p_ident, f));
            }
            emit_projector_mod(&mut w, &parsed.vis, &mod_name, &proj_names);
            for (idx, proj_ident, field) in active {
                emit_tuple_struct_projector(
                    &mut w,
                    &parsed.config,
                    &parsed.name,
                    &mod_name,
                    &proj_ident,
                    idx,
                    field,
                );
            }
        }
        ItemKind::Enum(variants) => {
            let mut proj_names = Vec::new();
            let mut active = Vec::new();
            for v in variants {
                if v.attr.skip {
                    continue;
                }
                proj_names.push(v.name.clone());
                active.push(v);
            }
            emit_projector_mod(&mut w, &parsed.vis, &mod_name, &proj_names);
            for variant in active {
                emit_enum_projector(&mut w, &parsed.config, &parsed.name, &mod_name, variant);
            }
        }
    }
    w.finish()
}

fn emit_projector_mod(
    w: &mut TokenWriter,
    vis: &[TokenTree],
    mod_name: &Ident,
    proj_names: &[Ident],
) {
    w.push_str("#[allow(missing_docs, dead_code, non_snake_case, unreachable_pub)] ");
    w.push_slice(vis);
    w.push_str("mod ");
    w.push_ident(mod_name);
    w.push_str(" ");
    w.push_grouped(Delimiter::Brace, |body| {
        for p in proj_names {
            body.push_str("/// Field or variant projector generated by `#[derive(Projectors)]`.\n");
            body.push_str("pub struct ");
            body.push_ident(p);
            body.push_str(";");
        }
    });
}

fn emit_call_signature(body: &mut TokenWriter, target_name: &Ident) {
    body.push_str("; #[inline(always)] fn call");
    body.push_grouped(Delimiter::Parenthesis, |args| {
        args.push_str("input: &'__fy_a ");
        args.push_ident(target_name);
    });
    body.push_str(" -> Self::Output ");
}

fn emit_named_struct_projector(
    w: &mut TokenWriter,
    config: &RefineConfig,
    struct_name: &Ident,
    mod_name: &Ident,
    proj_ident: &Ident,
    field: &NamedField,
) {
    let is_copy = field.attr.is_copy.unwrap_or(false);
    if config.is_const {
        w.push_str("const ");
    }
    w.push_str("impl<'__fy_a> ");
    w.push_slice(&config.crate_path);
    w.push_str("::function::Function<&'__fy_a ");
    w.push_ident(struct_name);
    w.push_str("> for ");
    w.push_ident(mod_name);
    w.push_str("::");
    w.push_ident(proj_ident);
    w.push_str(" ");
    w.push_grouped(Delimiter::Brace, |body| {
        body.push_str("type Output = ");
        if !is_copy {
            body.push_str("&'__fy_a ");
        }
        body.push_slice(&field.ty);
        emit_call_signature(body, struct_name);
        body.push_grouped(Delimiter::Brace, |fn_body| {
            if !is_copy {
                fn_body.push_str("&");
            }
            fn_body.push_str("input.");
            fn_body.push_ident(&field.name);
        });
    });

    w.push_str(
        "// SAFETY: Compile-time struct field offset projection without Deref or method calls.\n",
    );
    w.push_str("unsafe impl<'__fy_a> ");
    w.push_slice(&config.crate_path);
    w.push_str("::function::SafeFunction<&'__fy_a ");
    w.push_ident(struct_name);
    w.push_str("> for ");
    w.push_ident(mod_name);
    w.push_str("::");
    w.push_ident(proj_ident);
    w.push_str(" {}");
}

fn emit_tuple_struct_projector(
    w: &mut TokenWriter,
    config: &RefineConfig,
    struct_name: &Ident,
    mod_name: &Ident,
    proj_ident: &Ident,
    field_idx: usize,
    field: &TupleField,
) {
    let is_copy = field.attr.is_copy.unwrap_or(false);
    if config.is_const {
        w.push_str("const ");
    }
    w.push_str("impl<'__fy_a> ");
    w.push_slice(&config.crate_path);
    w.push_str("::function::Function<&'__fy_a ");
    w.push_ident(struct_name);
    w.push_str("> for ");
    w.push_ident(mod_name);
    w.push_str("::");
    w.push_ident(proj_ident);
    w.push_str(" ");
    w.push_grouped(Delimiter::Brace, |body| {
        body.push_str("type Output = ");
        if !is_copy {
            body.push_str("&'__fy_a ");
        }
        body.push_slice(&field.ty);
        emit_call_signature(body, struct_name);
        body.push_grouped(Delimiter::Brace, |fn_body| {
            if !is_copy {
                fn_body.push_str("&");
            }
            fn_body.push_str("input.");
            fn_body.push_index(field_idx, struct_name.span());
        });
    });

    w.push_str("// SAFETY: Compile-time tuple index projection without Deref or method calls.\n");
    w.push_str("unsafe impl<'__fy_a> ");
    w.push_slice(&config.crate_path);
    w.push_str("::function::SafeFunction<&'__fy_a ");
    w.push_ident(struct_name);
    w.push_str("> for ");
    w.push_ident(mod_name);
    w.push_str("::");
    w.push_ident(proj_ident);
    w.push_str(" {}");
}

fn emit_enum_projector(
    w: &mut TokenWriter,
    config: &RefineConfig,
    enum_name: &Ident,
    mod_name: &Ident,
    variant: &VariantDef,
) {
    let variant_copy_default = variant.attr.is_copy.unwrap_or(false);
    if config.is_const {
        w.push_str("const ");
    }
    w.push_str("impl<'__fy_a> ");
    w.push_slice(&config.crate_path);
    w.push_str("::function::Function<&'__fy_a ");
    w.push_ident(enum_name);
    w.push_str("> for ");
    w.push_ident(mod_name);
    w.push_str("::");
    w.push_ident(&variant.name);
    w.push_str(" ");
    w.push_grouped(Delimiter::Brace, |body| {
        body.push_str("type Output = ::core::option::Option<");
        match &variant.kind {
            VariantKind::Unit => body.push_str("()"),
            VariantKind::Named(fields) => {
                let mut active = Vec::new();
                for f in fields {
                    if !f.attr.skip {
                        active.push(f);
                    }
                }
                if active.len() == 1 {
                    let is_copy = active[0].attr.is_copy.unwrap_or(variant_copy_default);
                    if !is_copy {
                        body.push_str("&'__fy_a ");
                    }
                    body.push_slice(&active[0].ty);
                } else {
                    body.push_grouped(Delimiter::Parenthesis, |tup| {
                        for (i, f) in active.iter().enumerate() {
                            if i > 0 {
                                tup.push_str(", ");
                            }
                            let is_copy = f.attr.is_copy.unwrap_or(variant_copy_default);
                            if !is_copy {
                                tup.push_str("&'__fy_a ");
                            }
                            tup.push_slice(&f.ty);
                        }
                    });
                }
            }
            VariantKind::Tuple(fields) => {
                let mut active = Vec::new();
                for (idx, f) in fields.iter().enumerate() {
                    if !f.attr.skip {
                        active.push((idx, f));
                    }
                }
                if active.len() == 1 {
                    let is_copy = active[0].1.attr.is_copy.unwrap_or(variant_copy_default);
                    if !is_copy {
                        body.push_str("&'__fy_a ");
                    }
                    body.push_slice(&active[0].1.ty);
                } else {
                    body.push_grouped(Delimiter::Parenthesis, |tup| {
                        for (i, (_, f)) in active.iter().enumerate() {
                            if i > 0 {
                                tup.push_str(", ");
                            }
                            let is_copy = f.attr.is_copy.unwrap_or(variant_copy_default);
                            if !is_copy {
                                tup.push_str("&'__fy_a ");
                            }
                            tup.push_slice(&f.ty);
                        }
                    });
                }
            }
        }
        body.push_str(">");
        emit_call_signature(body, enum_name);
        body.push_grouped(Delimiter::Brace, |fn_body| {
            fn_body.push_str("#[allow(unreachable_patterns)] match input ");
            fn_body.push_grouped(Delimiter::Brace, |arms| {
                arms.push_ident(enum_name);
                arms.push_str("::");
                arms.push_ident(&variant.name);
                match &variant.kind {
                    VariantKind::Unit => {
                        arms.push_str(" => ::core::option::Option::Some(()),");
                    }
                    VariantKind::Named(fields) => {
                        let active: Vec<&NamedField> =
                            fields.iter().filter(|f| !f.attr.skip).collect();
                        let has_skipped = active.len() != fields.len();
                        arms.push_grouped(Delimiter::Brace, |pat| {
                            for (i, f) in active.iter().enumerate() {
                                if i > 0 {
                                    pat.push_str(", ");
                                }
                                pat.push_ident(&f.name);
                            }
                            if has_skipped {
                                if !active.is_empty() {
                                    pat.push_str(", ");
                                }
                                pat.push_str("..");
                            }
                        });
                        arms.push_str(" => ::core::option::Option::Some");
                        arms.push_grouped(Delimiter::Parenthesis, |ret| {
                            if active.len() == 1 {
                                let is_copy =
                                    active[0].attr.is_copy.unwrap_or(variant_copy_default);
                                if is_copy {
                                    ret.push_str("*");
                                }
                                ret.push_ident(&active[0].name);
                            } else {
                                ret.push_grouped(Delimiter::Parenthesis, |tup| {
                                    for (i, f) in active.iter().enumerate() {
                                        if i > 0 {
                                            tup.push_str(", ");
                                        }
                                        let is_copy =
                                            f.attr.is_copy.unwrap_or(variant_copy_default);
                                        if is_copy {
                                            tup.push_str("*");
                                        }
                                        tup.push_ident(&f.name);
                                    }
                                });
                            }
                        });
                        arms.push_str(",");
                    }
                    VariantKind::Tuple(fields) => {
                        let active: Vec<(usize, &TupleField)> = fields
                            .iter()
                            .enumerate()
                            .filter(|(_, f)| !f.attr.skip)
                            .collect();
                        arms.push_grouped(Delimiter::Parenthesis, |pat| {
                            for (i, f) in fields.iter().enumerate() {
                                if i > 0 {
                                    pat.push_str(", ");
                                }
                                if f.attr.skip {
                                    pat.push_str("_");
                                } else {
                                    pat.push_str(&format!("__fy_v{i}"));
                                }
                            }
                        });
                        arms.push_str(" => ::core::option::Option::Some");
                        arms.push_grouped(Delimiter::Parenthesis, |ret| {
                            if active.len() == 1 {
                                let (idx, f) = active[0];
                                let is_copy = f.attr.is_copy.unwrap_or(variant_copy_default);
                                if is_copy {
                                    ret.push_str("*");
                                }
                                ret.push_str(&format!("__fy_v{idx}"));
                            } else {
                                ret.push_grouped(Delimiter::Parenthesis, |tup| {
                                    for (pos, (idx, f)) in active.iter().enumerate() {
                                        if pos > 0 {
                                            tup.push_str(", ");
                                        }
                                        let is_copy =
                                            f.attr.is_copy.unwrap_or(variant_copy_default);
                                        if is_copy {
                                            tup.push_str("*");
                                        }
                                        tup.push_str(&format!("__fy_v{idx}"));
                                    }
                                });
                            }
                        });
                        arms.push_str(",");
                    }
                }
                arms.push_str(" _ => ::core::option::Option::None");
            });
        });
    });

    w.push_str(
        "// SAFETY: Compile-time pure enum pattern destructuring without Deref or method calls.\n",
    );
    w.push_str("unsafe impl<'__fy_a> ");
    w.push_slice(&config.crate_path);
    w.push_str("::function::SafeFunction<&'__fy_a ");
    w.push_ident(enum_name);
    w.push_str("> for ");
    w.push_ident(mod_name);
    w.push_str("::");
    w.push_ident(&variant.name);
    w.push_str(" {}");
}

fn parse_item(input: TokenStream) -> Result<ParsedItem, ParseError> {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut idx = 0usize;

    let (config, _) = parse_attributes(&tokens, &mut idx, true)?;
    let vis = parse_visibility(&tokens, &mut idx);

    let kw = match tokens.get(idx) {
        Some(TokenTree::Ident(id)) => {
            idx += 1;
            id.clone()
        }
        Some(other) => {
            return Err(ParseError::new(
                other.span(),
                "expected `struct` or `enum` keyword",
            ));
        }
        None => {
            return Err(ParseError::new(
                Span::call_site(),
                "expected a `struct` or `enum` definition",
            ));
        }
    };

    let kw_str = kw.to_string();
    if kw_str != "struct" && kw_str != "enum" {
        return Err(ParseError::new(
            kw.span(),
            "fy-refine derive macros only support `struct` and `enum` definitions",
        ));
    }

    let name = match tokens.get(idx) {
        Some(TokenTree::Ident(id)) => {
            idx += 1;
            id.clone()
        }
        Some(other) => {
            return Err(ParseError::new(
                other.span(),
                "expected type identifier name",
            ));
        }
        None => return Err(ParseError::new(kw.span(), "missing type name")),
    };

    match tokens.get(idx) {
        Some(TokenTree::Punct(p)) if p.as_char() == '<' => {
            return Err(ParseError::new(
                p.span(),
                "fy-refine-macro derive macros do not yet support generic type or lifetime parameters",
            ));
        }
        Some(TokenTree::Ident(id)) if id.to_string() == "where" => {
            return Err(ParseError::new(
                id.span(),
                "fy-refine-macro derive macros do not yet support `where` clauses",
            ));
        }
        _ => {}
    }

    let kind = if kw_str == "struct" {
        match tokens.get(idx) {
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                let fields = parse_named_fields(&inner)?;
                if fields.is_empty() {
                    ItemKind::StructUnit
                } else {
                    ItemKind::StructNamed(fields)
                }
            }
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                let fields = parse_tuple_fields(&inner)?;
                if fields.is_empty() {
                    return Err(ParseError::new(
                        g.span(),
                        "empty tuple structs are not supported; use a unit struct instead",
                    ));
                }
                ItemKind::StructTuple(fields)
            }
            Some(TokenTree::Punct(p)) if p.as_char() == ';' => ItemKind::StructUnit,
            Some(other) => {
                return Err(ParseError::new(
                    other.span(),
                    "unrecognized struct body form",
                ));
            }
            None => return Err(ParseError::new(name.span(), "missing struct body")),
        }
    } else {
        match tokens.get(idx) {
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                let variants = parse_enum_variants(&inner)?;
                if variants.is_empty() {
                    return Err(ParseError::new(
                        g.span(),
                        "empty enums (zero variants) are not supported",
                    ));
                }
                ItemKind::Enum(variants)
            }
            Some(other) => {
                return Err(ParseError::new(
                    other.span(),
                    "expected enum body `{ ... }`",
                ));
            }
            None => return Err(ParseError::new(name.span(), "missing enum body")),
        }
    };

    Ok(ParsedItem {
        vis,
        name,
        config,
        kind,
    })
}

fn parse_attributes(
    tokens: &[TokenTree],
    idx: &mut usize,
    allow_refine: bool,
) -> Result<(RefineConfig, ProjectorAttr), ParseError> {
    let mut config = RefineConfig::default();
    let mut proj = ProjectorAttr::default();

    while matches!(tokens.get(*idx), Some(TokenTree::Punct(p)) if p.as_char() == '#') {
        let hash_span = tokens[*idx].span();
        if let Some(TokenTree::Group(g)) = tokens
            .get(*idx + 1)
            .filter(|tt| matches!(tt, TokenTree::Group(g) if g.delimiter() == Delimiter::Bracket))
        {
            *idx += 2;
            let inner: Vec<TokenTree> = g.stream().into_iter().collect();
            let Some(TokenTree::Ident(attr_name)) = inner.first() else {
                continue;
            };
            let attr_str = attr_name.to_string();
            if attr_str == "refine" {
                if !allow_refine {
                    return Err(ParseError::new(
                        attr_name.span(),
                        "`#[refine(...)]` is only allowed on top-level struct or enum items",
                    ));
                }
                let Some(TokenTree::Group(args)) = inner.get(1) else {
                    return Err(ParseError::new(
                        attr_name.span(),
                        "expected `#[refine(...)]` argument list",
                    ));
                };
                if args.delimiter() != Delimiter::Parenthesis || inner.len() > 2 {
                    return Err(ParseError::new(
                        args.span(),
                        "invalid `#[refine(...)]` attribute syntax",
                    ));
                }
                parse_refine_args(&args.stream().into_iter().collect::<Vec<_>>(), &mut config)?;
            } else if attr_str == "projector" {
                if allow_refine {
                    return Err(ParseError::new(
                        attr_name.span(),
                        "`#[projector(...)]` is only allowed on fields or enum variants",
                    ));
                }
                let Some(TokenTree::Group(args)) = inner.get(1) else {
                    return Err(ParseError::new(
                        attr_name.span(),
                        "expected `#[projector(...)]` argument list",
                    ));
                };
                if args.delimiter() != Delimiter::Parenthesis || inner.len() > 2 {
                    return Err(ParseError::new(
                        args.span(),
                        "invalid `#[projector(...)]` attribute syntax",
                    ));
                }
                parse_projector_args(&args.stream().into_iter().collect::<Vec<_>>(), &mut proj)?;
            }
        } else {
            return Err(ParseError::new(hash_span, "invalid attribute syntax"));
        }
    }

    Ok((config, proj))
}

fn parse_refine_args(tokens: &[TokenTree], config: &mut RefineConfig) -> Result<(), ParseError> {
    for chunk in split_top_level_commas(tokens) {
        if chunk.is_empty() {
            continue;
        }
        let Some(TokenTree::Ident(key)) = chunk.first() else {
            return Err(ParseError::new(
                chunk[0].span(),
                "expected `refine` attribute key identifier",
            ));
        };
        match key.to_string().as_str() {
            "const" if chunk.len() == 1 => config.is_const = true,
            "crate" if matches!(chunk.get(1), Some(TokenTree::Punct(p)) if p.as_char() == '=') => {
                if chunk.len() <= 2 {
                    return Err(ParseError::new(key.span(), "missing path after `crate =`"));
                }
                config.crate_path = chunk[2..].to_vec();
            }
            "mod" if matches!(chunk.get(1), Some(TokenTree::Punct(p)) if p.as_char() == '=') => {
                if let (3, Some(TokenTree::Ident(mod_id))) = (chunk.len(), chunk.get(2)) {
                    config.mod_name = Some(mod_id.clone());
                } else {
                    return Err(ParseError::new(
                        key.span(),
                        "`mod = ...` accepts a single module identifier",
                    ));
                }
            }
            _ => {
                return Err(ParseError::new(
                    key.span(),
                    "unsupported `#[refine(...)]` option; expected `const`, `crate = ...`, or `mod = ...`",
                ));
            }
        }
    }
    Ok(())
}

fn parse_projector_args(tokens: &[TokenTree], proj: &mut ProjectorAttr) -> Result<(), ParseError> {
    for chunk in split_top_level_commas(tokens) {
        if chunk.is_empty() {
            continue;
        }
        if let (1, Some(TokenTree::Ident(key))) = (chunk.len(), chunk.first()) {
            match key.to_string().as_str() {
                "copy" => proj.is_copy = Some(true),
                "ref" => proj.is_copy = Some(false),
                "skip" => proj.skip = true,
                _ => {
                    return Err(ParseError::new(
                        key.span(),
                        "unsupported `#[projector(...)]` option; expected `copy`, `ref`, or `skip`",
                    ));
                }
            }
        } else {
            return Err(ParseError::new(
                chunk[0].span(),
                "invalid `#[projector(...)]` option syntax",
            ));
        }
    }
    Ok(())
}

fn parse_visibility(tokens: &[TokenTree], idx: &mut usize) -> Vec<TokenTree> {
    let mut vis = Vec::new();
    if matches!(tokens.get(*idx), Some(TokenTree::Ident(id)) if id.to_string() == "pub") {
        vis.push(tokens[*idx].clone());
        *idx += 1;
        if matches!(
            tokens.get(*idx),
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis
        ) {
            vis.push(tokens[*idx].clone());
            *idx += 1;
        }
    }
    vis
}

fn parse_named_fields(tokens: &[TokenTree]) -> Result<Vec<NamedField>, ParseError> {
    let mut fields = Vec::new();
    for chunk in split_top_level_commas(tokens) {
        if chunk.is_empty() {
            continue;
        }
        let mut idx = 0usize;
        let (_, attr) = parse_attributes(&chunk, &mut idx, false)?;
        let _vis = parse_visibility(&chunk, &mut idx);

        let name = match chunk.get(idx) {
            Some(TokenTree::Ident(id)) => {
                idx += 1;
                id.clone()
            }
            Some(other) => return Err(ParseError::new(other.span(), "expected struct field name")),
            None => {
                return Err(ParseError::new(
                    chunk[0].span(),
                    "missing struct field name",
                ));
            }
        };

        match chunk.get(idx) {
            Some(TokenTree::Punct(p)) if p.as_char() == ':' => idx += 1,
            Some(other) => {
                return Err(ParseError::new(
                    other.span(),
                    "expected `:` after struct field name",
                ));
            }
            None => return Err(ParseError::new(name.span(), "missing `: Type` on field")),
        }

        if idx >= chunk.len() {
            return Err(ParseError::new(name.span(), "missing field type"));
        }

        fields.push(NamedField {
            name,
            ty: chunk[idx..].to_vec(),
            attr,
        });
    }
    Ok(fields)
}

fn parse_tuple_fields(tokens: &[TokenTree]) -> Result<Vec<TupleField>, ParseError> {
    let mut fields = Vec::new();
    for chunk in split_top_level_commas(tokens) {
        if chunk.is_empty() {
            continue;
        }
        let mut idx = 0usize;
        let (_, attr) = parse_attributes(&chunk, &mut idx, false)?;
        let _vis = parse_visibility(&chunk, &mut idx);
        if idx >= chunk.len() {
            return Err(ParseError::new(chunk[0].span(), "missing tuple field type"));
        }
        fields.push(TupleField {
            ty: chunk[idx..].to_vec(),
            attr,
        });
    }
    Ok(fields)
}

fn parse_enum_variants(tokens: &[TokenTree]) -> Result<Vec<VariantDef>, ParseError> {
    let mut variants = Vec::new();
    for chunk in split_top_level_commas(tokens) {
        if chunk.is_empty() {
            continue;
        }
        let mut idx = 0usize;
        let (_, attr) = parse_attributes(&chunk, &mut idx, false)?;
        let name = match chunk.get(idx) {
            Some(TokenTree::Ident(id)) => {
                idx += 1;
                id.clone()
            }
            Some(other) => return Err(ParseError::new(other.span(), "expected enum variant name")),
            None => {
                return Err(ParseError::new(
                    chunk[0].span(),
                    "missing enum variant name",
                ));
            }
        };

        let kind = match chunk.get(idx) {
            None => VariantKind::Unit,
            Some(TokenTree::Punct(p)) if p.as_char() == '=' => VariantKind::Unit,
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                VariantKind::Named(parse_named_fields(&inner)?)
            }
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                VariantKind::Tuple(parse_tuple_fields(&inner)?)
            }
            Some(other) => {
                return Err(ParseError::new(
                    other.span(),
                    "unrecognized enum variant definition",
                ));
            }
        };

        variants.push(VariantDef { name, attr, kind });
    }
    Ok(variants)
}

fn split_top_level_commas(tokens: &[TokenTree]) -> Vec<Vec<TokenTree>> {
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut angle_depth = 0usize;
    let mut i = 0usize;

    while i < tokens.len() {
        if matches!(
            (tokens.get(i), tokens.get(i + 1)),
            (Some(TokenTree::Punct(p1)), Some(TokenTree::Punct(p2)))
                if matches!((p1.as_char(), p2.as_char()), ('-', '>') | ('=', '>'))
        ) {
            current.push(tokens[i].clone());
            current.push(tokens[i + 1].clone());
            i += 2;
            continue;
        }

        let tt = &tokens[i];
        match tt {
            TokenTree::Punct(p) if p.as_char() == '<' => {
                angle_depth += 1;
                current.push(tt.clone());
            }
            TokenTree::Punct(p) if p.as_char() == '>' => {
                angle_depth = angle_depth.saturating_sub(1);
                current.push(tt.clone());
            }
            TokenTree::Punct(p) if p.as_char() == ',' && angle_depth == 0 => {
                chunks.push(core::mem::take(&mut current));
            }
            _ => current.push(tt.clone()),
        }
        i += 1;
    }

    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn to_snake_case(name: &str) -> String {
    let raw = name.strip_prefix("r#").unwrap_or(name);
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len() + 4);
    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                let next_lower = chars.get(i + 1).is_some_and(|c| c.is_lowercase());
                if prev.is_lowercase()
                    || prev.is_ascii_digit()
                    || (prev.is_uppercase() && next_lower)
                {
                    out.push('_');
                }
            }
            for lc in ch.to_lowercase() {
                out.push(lc);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn to_pascal_case(name: &str) -> String {
    let raw = name.strip_prefix("r#").unwrap_or(name);
    let mut out = String::with_capacity(raw.len());
    for part in raw.split('_') {
        if part.is_empty() {
            continue;
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            for uc in first.to_uppercase() {
                out.push(uc);
            }
            out.push_str(chars.as_str());
        }
    }
    out
}

fn escape_keyword(ident: &str) -> String {
    match ident {
        "as" | "break" | "const" | "continue" | "else" | "enum" | "extern" | "false" | "fn"
        | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut"
        | "pub" | "ref" | "return" | "static" | "struct" | "trait" | "true" | "type" | "unsafe"
        | "use" | "where" | "while" | "async" | "await" | "dyn" | "abstract" | "become" | "box"
        | "do" | "final" | "macro" | "override" | "priv" | "try" | "typeof" | "unsized"
        | "virtual" | "yield" | "gen" => format!("r#{ident}"),
        _ => ident.to_string(),
    }
}

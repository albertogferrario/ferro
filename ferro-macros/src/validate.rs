//! Derive macro for declarative struct validation.
//!
//! Generates `Validatable` trait implementation from field attributes.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Expr, Fields, Ident, Lit, Token,
};

/// Returns the token stream for the ferro crate path: `::ferro`
fn ferro() -> TokenStream2 {
    quote!(::ferro)
}

/// A single validation rule parsed from attributes
#[derive(Debug, Clone)]
struct ParsedRule {
    /// Rule name (e.g., "required", "email", "min")
    name: String,
    /// Optional arguments (e.g., 8 for min(8))
    args: Vec<RuleArg>,
}

/// An argument to a validation rule
#[derive(Debug, Clone)]
enum RuleArg {
    /// Integer argument (e.g., min(8))
    Int(i64),
    /// Float argument (e.g., min(8.5))
    Float(f64),
    /// String argument (e.g., required_if("field", "value"))
    String(String),
    /// Identifier argument (e.g., same(other_field))
    Ident(String),
}

impl ToTokens for RuleArg {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            RuleArg::Int(n) => n.to_tokens(tokens),
            RuleArg::Float(n) => n.to_tokens(tokens),
            RuleArg::String(s) => s.to_tokens(tokens),
            RuleArg::Ident(s) => {
                let ident = quote::format_ident!("{}", s);
                ident.to_tokens(tokens)
            }
        }
    }
}

/// Field with its parsed rules
struct FieldRules {
    name: String,
    rules: Vec<ParsedRule>,
}

/// Generate Validatable implementation for a struct
pub fn validate_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract fields from struct
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "ValidateRules only supports named structs",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "ValidateRules only supports structs")
                .to_compile_error()
                .into();
        }
    };

    // Parse rules from each field's attributes
    let mut field_rules = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let rules = parse_field_rules(field);

        if !rules.is_empty() {
            field_rules.push(FieldRules {
                name: field_name,
                rules,
            });
        }
    }

    let ferro = ferro();

    // Generate code
    let validate_impl = generate_validate_impl(&ferro, &field_rules);
    let rules_impl = generate_rules_impl(&ferro, &field_rules);

    let expanded = quote! {
        impl #ferro::validation::Validatable for #name
        where
            Self: ::serde::Serialize,
        {
            fn validate(&self) -> ::std::result::Result<(), #ferro::validation::ValidationError> {
                #validate_impl
            }

            fn validation_rules() -> ::std::vec::Vec<(&'static str, ::std::vec::Vec<::std::boxed::Box<dyn #ferro::validation::Rule>>)> {
                #rules_impl
            }
        }
    };

    TokenStream::from(expanded)
}

/// Parse #[rule(...)] attributes from a field
fn parse_field_rules(field: &syn::Field) -> Vec<ParsedRule> {
    let mut rules = Vec::new();

    for attr in &field.attrs {
        if !attr.path().is_ident("rule") {
            continue;
        }

        // Parse the attribute content: #[rule(required, email, min(8))]
        let result: Result<Punctuated<Expr, Token![,]>, _> =
            attr.parse_args_with(Punctuated::parse_terminated);

        if let Ok(exprs) = result {
            for expr in exprs {
                if let Some(rule) = parse_rule_expr(&expr) {
                    rules.push(rule);
                }
            }
        }
    }

    rules
}

/// Parse a single rule expression (e.g., `required` or `min(8)`)
fn parse_rule_expr(expr: &Expr) -> Option<ParsedRule> {
    match expr {
        // Simple rule: required, email, string, etc.
        Expr::Path(path) => {
            let name = path.path.get_ident()?.to_string();
            Some(ParsedRule {
                name,
                args: Vec::new(),
            })
        }
        // Rule with args: min(8), between(1, 100), required_if("field", "value")
        Expr::Call(call) => {
            let name = if let Expr::Path(path) = call.func.as_ref() {
                path.path.get_ident()?.to_string()
            } else {
                return None;
            };

            let args: Vec<RuleArg> = call.args.iter().filter_map(parse_rule_arg).collect();

            Some(ParsedRule { name, args })
        }
        _ => None,
    }
}

/// Parse a single rule argument
fn parse_rule_arg(expr: &Expr) -> Option<RuleArg> {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Int(n) => Some(RuleArg::Int(n.base10_parse().ok()?)),
            Lit::Float(n) => Some(RuleArg::Float(n.base10_parse().ok()?)),
            Lit::Str(s) => Some(RuleArg::String(s.value())),
            _ => None,
        },
        Expr::Path(path) => {
            let ident = path.path.get_ident()?;
            Some(RuleArg::Ident(ident.to_string()))
        }
        _ => None,
    }
}

/// Generate the validate() method implementation
fn generate_validate_impl(ferro: &TokenStream2, field_rules: &[FieldRules]) -> TokenStream2 {
    if field_rules.is_empty() {
        return quote! { Ok(()) };
    }

    // Build rule applications for each field
    let mut field_validations = Vec::new();

    for fr in field_rules {
        let field_name = &fr.name;

        let rule_calls: Vec<TokenStream2> = fr
            .rules
            .iter()
            .map(|r| generate_rule_call(ferro, r))
            .collect();

        field_validations.push(quote! {
            validator = validator.rules(#field_name, #ferro::rules![#(#rule_calls),*]);
        });
    }

    quote! {
        // Serialize self to JSON for validation
        let data = #ferro::serde_json::to_value(self)
            .map_err(|e| {
                let mut err = #ferro::validation::ValidationError::new();
                err.add("_struct", format!("Failed to serialize: {}", e));
                err
            })?;

        // Build validator with rules
        let mut validator = #ferro::validation::Validator::new(&data);
        #(#field_validations)*

        validator.validate()
    }
}

/// Generate a single rule function call
fn generate_rule_call(ferro: &TokenStream2, rule: &ParsedRule) -> TokenStream2 {
    let rule_fn = Ident::new(&rule.name, proc_macro2::Span::call_site());

    if rule.args.is_empty() {
        quote! { #ferro::validation::#rule_fn() }
    } else {
        let args = &rule.args;
        quote! { #ferro::validation::#rule_fn(#(#args),*) }
    }
}

/// Generate the validation_rules() method implementation
fn generate_rules_impl(ferro: &TokenStream2, field_rules: &[FieldRules]) -> TokenStream2 {
    if field_rules.is_empty() {
        return quote! { ::std::vec::Vec::new() };
    }

    let mut rule_entries = Vec::new();

    for fr in field_rules {
        let field_name = &fr.name;

        let rule_calls: Vec<TokenStream2> = fr
            .rules
            .iter()
            .map(|rule| {
                let call = generate_rule_call(ferro, rule);
                quote! {
                    ::std::boxed::Box::new(#call) as ::std::boxed::Box<dyn #ferro::validation::Rule>
                }
            })
            .collect();

        rule_entries.push(quote! {
            (#field_name, ::std::vec![#(#rule_calls),*])
        });
    }

    quote! {
        ::std::vec![#(#rule_entries),*]
    }
}

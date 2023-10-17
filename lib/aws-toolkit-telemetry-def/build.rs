use convert_case::{
    Case,
    Casing,
};
use quote::{
    format_ident,
    quote,
};

const DEF: &str = include_str!("./def.json");

#[derive(Debug, serde::Deserialize)]
struct TypeDef {
    name: String,
    r#type: Option<String>,
    description: String,
}

#[derive(Debug, serde::Deserialize)]
struct MetricDef {
    name: String,
    description: String,
    metadata: Option<Vec<MetricMetadata>>,
}

#[derive(Debug, serde::Deserialize)]
struct MetricMetadata {
    r#type: String,
    required: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
struct Def {
    types: Vec<TypeDef>,
    metrics: Vec<MetricDef>,
}

fn main() {
    println!("cargo:rerun-if-changed=def.json");

    let outdir = std::env::var("OUT_DIR").unwrap();

    let data = serde_json::from_str::<Def>(DEF).unwrap();

    let mut out = String::new();

    out.push_str("pub mod types {");
    for t in data.types {
        let name = format_ident!("{}", t.name.to_case(Case::Pascal));
        let r#type = match t.r#type.as_deref() {
            Some("string") | None => quote!(String),
            Some("int") => quote!(i64),
            Some("double") => quote!(f64),
            Some("boolean") => quote!(bool),
            Some(other) => panic!("{}", other),
        };
        let description = t.description;

        let rust_type = quote::quote!(
            #[doc = #description]
            pub type #name = #r#type;
        );

        out.push_str(&rust_type.to_string());
    }
    out.push('}');

    out.push_str("pub mod metrics {");
    for m in data.metrics {
        let name = format_ident!("{}", m.name.to_case(Case::Pascal));
        let description = m.description;

        let mut fields = Vec::new();
        for field in m.metadata.unwrap_or_default() {
            let field_name = format_ident!("{}", &field.r#type.to_case(Case::Snake));
            let ty_name = format_ident!("{}", field.r#type.to_case(Case::Pascal));
            let ty = if field.required.unwrap_or_default() {
                quote!(crate::types::#ty_name)
            } else {
                quote!(Option<crate::types::#ty_name>)
            };

            fields.push(quote!(
                pub #field_name: #ty
            ));
        }

        let rust_type = quote::quote!(
            #[doc = #description]
            pub struct #name {
                #( #fields, )*
            }
        );

        out.push_str(&rust_type.to_string());
    }
    out.push('}');

    // write an empty file to the output directory
    std::fs::write(format!("{}/mod.rs", outdir), out).unwrap();
}

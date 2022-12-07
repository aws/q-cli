use convert_case::{
    Case,
    Casing,
};

fn queries() -> Vec<String> {
    std::fs::read_dir("queries")
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap();
            let name = name.split('.').next().unwrap();
            name.to_owned()
        })
        .collect()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=schema/schema.graphql");

    let ignore_lints = ["clippy::derive_partial_eq_without_eq"];
    let imports = ["super::*", "graphql_client::GraphQLQuery"];

    let mut out_str: String = "mod gql {\n".into();

    for lint in ignore_lints {
        out_str.push_str(&format!("#![allow({lint})]\n"));
    }

    for import in imports {
        out_str.push_str(&format!("use {import};\n"));
    }

    for query_name in queries() {
        println!("cargo:rerun-if-changed=queries/{query_name}.graphql");

        let snake_case_name = query_name.to_case(Case::Snake);
        let pascal_case_name = query_name.to_case(Case::Pascal);

        let item = format!("

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = \"schema/schema.graphql\",
    query_path = \"queries/{query_name}.graphql\",
    response_derives = \"Debug, Clone, PartialEq\",
    variables_derives = \"Debug, Clone, PartialEq, Default\",
    normalization = \"rust\"
)]
pub struct {pascal_case_name};
    
pub async fn {snake_case_name}_request(variables: {snake_case_name}::Variables) -> fig_request::Result<{snake_case_name}::ResponseData> {{
    let request_body = {pascal_case_name}::build_query(variables);
    fig_request::Request::post(\"/graphql\")
        .auth()
        .body(request_body)
        .graphql()
        .await
}}

#[macro_export]
macro_rules! {snake_case_name} {{
    ($( $arg:ident $( : $val:expr )? ),* $(, ..$default:expr )?) => {{{{
        let variables = $crate::{snake_case_name}::Variables {{ $($arg $(: $val.into())?),* $(, ..$default)? }};
        $crate::{snake_case_name}_request(variables)
    }}}};
    // Allow for trailing comma
    ($( $arg:ident $( : $val:expr )? , )* $( ..$default:expr, )?) => {{{{
        $crate::{snake_case_name}!($( $arg $(: $val)?),* $(, ..$default)?)
    }}}};
}}

");

        out_str.push_str(&item);
    }

    out_str.push_str("}\npub use gql::*;\n");

    std::fs::write(format!("{}/queries.rs", std::env::var("OUT_DIR").unwrap()), out_str).unwrap();
}

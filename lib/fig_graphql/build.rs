use convert_case::{
    Case,
    Casing,
};

fn queries() -> Vec<(String, String)> {
    std::fs::read_dir("queries")
        .unwrap()
        .flat_map(|entry| {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap();
            let file_name = name.split('.').next().unwrap().to_owned();

            // Read the file and get all the queries
            let contents = std::fs::read_to_string(path).unwrap();
            regex::Regex::new(r#"(?m)^(query|mutation)\s+(\w+)"#)
                .unwrap()
                .captures_iter(&contents)
                .map(|cap| (file_name.clone(), cap[2].to_owned()))
                .collect::<Vec<_>>()
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

    for (query_file_name, query_name) in queries() {
        println!("cargo:rerun-if-changed=queries/{query_file_name}.graphql");

        let snake_case_name = query_name.to_case(Case::Snake);

        let item = format!("

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = \"schema/schema.graphql\",
    query_path = \"queries/{query_file_name}.graphql\",
    response_derives = \"Debug, Clone, PartialEq\",
    variables_derives = \"Debug, Clone, PartialEq, Default\",
    normalization = \"rust\"
)]
pub struct {query_name};

pub async fn {snake_case_name}_request(variables: {snake_case_name}::Variables) -> fig_request::Result<{snake_case_name}::ResponseData> {{
    let request_body = {query_name}::build_query(variables);
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

#[macro_export]
macro_rules! {snake_case_name}_query {{
    ($( $arg:ident $( : $val:expr )? ),* $(, ..$default:expr )?) => {{{{
        use $crate::GraphQLQuery;
        let variables = $crate::{snake_case_name}::Variables {{ $($arg $(: $val.into())?),* $(, ..$default)? }};
        $crate::{query_name}::build_query(variables)
    }}}};
    // Allow for trailing comma
    ($( $arg:ident $( : $val:expr )? , )* $( ..$default:expr, )?) => {{{{
        $crate::{snake_case_name}_query!($( $arg $(: $val)?),* $(, ..$default)?)
    }}}};
}}

");

        out_str.push_str(&item);
    }

    out_str.push_str("}\npub use gql::*;\n");

    std::fs::write(format!("{}/queries.rs", std::env::var("OUT_DIR").unwrap()), out_str).unwrap();
}

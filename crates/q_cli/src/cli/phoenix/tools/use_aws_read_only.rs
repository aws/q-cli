
pub const USE_AWS_READ_ONLY: &str = r#"
{
    "name": "use_aws_read_only",
    "description": "Make an AWS api call with the specified service, operation, and parameters. You may not create resources or perform any write or mutating actions. You may only use this tool to call read operations with names that start with: get, describe, list, search, batch_get.",
    "inputSchema": {
        "json": {
            "type": "object",
            "properties": {
                "service_name": {
                    "type": "string",
                    "description": "The name of the AWS service."
                },
                "operation_name": {
                    "type": "string",
                    "description": "The name of the operation to perform."
                },
                "parameters": {
                    "type": "object",
                    "description": "The parameters for the operation."
                },
                "region": {
                    "type": "string",
                    "description": "Region name for calling the operation on AWS."
                },
                "profile_name": {
                    "type": "string",
                    "description": "Optional: AWS profile name to use from ~/.aws/credentials. Defaults to default profile if not specified."
                },
                "label": {
                    "type": "string",
                    "description": "Human readable description of the api that is being called.",
                },
            },
            "required": [
                "region",
                "service_name",
                "operation_name",
                "parameters",
                "label
            ]
        }
    }
}
"#;


// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_test_generation_event(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::TestGenerationEvent,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("jobId").string(input.job_id.as_str());
    }
    {
        object.key("groupName").string(input.group_name.as_str());
    }
    if let Some(var_1) = &input.timestamp {
        object
            .key("timestamp")
            .date_time(var_1, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    if let Some(var_2) = &input.ide_category {
        object.key("ideCategory").string(var_2.as_str());
    }
    if let Some(var_3) = &input.programming_language {
        #[allow(unused_mut)]
        let mut object_4 = object.key("programmingLanguage").start_object();
        crate::protocol_serde::shape_programming_language::ser_programming_language(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.number_of_unit_test_cases_generated {
        object.key("numberOfUnitTestCasesGenerated").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_5).into()),
        );
    }
    if let Some(var_6) = &input.number_of_unit_test_cases_accepted {
        object.key("numberOfUnitTestCasesAccepted").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    if let Some(var_7) = &input.lines_of_code_generated {
        object.key("linesOfCodeGenerated").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_7).into()),
        );
    }
    if let Some(var_8) = &input.lines_of_code_accepted {
        object.key("linesOfCodeAccepted").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_8).into()),
        );
    }
    if let Some(var_9) = &input.chars_of_code_generated {
        object.key("charsOfCodeGenerated").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_9).into()),
        );
    }
    if let Some(var_10) = &input.chars_of_code_accepted {
        object.key("charsOfCodeAccepted").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_10).into()),
        );
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_code_fix_acceptance_event(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::CodeFixAcceptanceEvent,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("jobId").string(input.job_id.as_str());
    }
    if let Some(var_1) = &input.rule_id {
        object.key("ruleId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.detector_id {
        object.key("detectorId").string(var_2.as_str());
    }
    if let Some(var_3) = &input.finding_id {
        object.key("findingId").string(var_3.as_str());
    }
    if let Some(var_4) = &input.programming_language {
        #[allow(unused_mut)]
        let mut object_5 = object.key("programmingLanguage").start_object();
        crate::protocol_serde::shape_programming_language::ser_programming_language(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.lines_of_code_accepted {
        object.key("linesOfCodeAccepted").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    if let Some(var_7) = &input.chars_of_code_accepted {
        object.key("charsOfCodeAccepted").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_7).into()),
        );
    }
    Ok(())
}

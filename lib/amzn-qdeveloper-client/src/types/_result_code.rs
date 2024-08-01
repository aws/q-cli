// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// When writing a match expression against `ResultCode`, it is important to ensure
/// your code is forward-compatible. That is, if a match arm handles a case for a
/// feature that is supported by the service but has not been represented as an enum
/// variant in a current version of SDK, your code should continue to work when you
/// upgrade SDK to a future version in which the enum does include a variant for that
/// feature.
///
/// Here is an example of how you can make a match expression forward-compatible:
///
/// ```text
/// # let resultcode = unimplemented!();
/// match resultcode {
///     ResultCode::Inappropriate => { /* ... */ },
///     ResultCode::Llm => { /* ... */ },
///     ResultCode::NoAnswer => { /* ... */ },
///     ResultCode::Ood => { /* ... */ },
///     ResultCode::Pii => { /* ... */ },
///     ResultCode::Prohibited => { /* ... */ },
///     other @ _ if other.as_str() == "NewFeature" => { /* handles a case for `NewFeature` */ },
///     _ => { /* ... */ },
/// }
/// ```
/// The above code demonstrates that when `resultcode` represents
/// `NewFeature`, the execution path will lead to the second last match arm,
/// even though the enum does not contain a variant `ResultCode::NewFeature`
/// in the current version of SDK. The reason is that the variable `other`,
/// created by the `@` operator, is bound to
/// `ResultCode::Unknown(UnknownVariantValue("NewFeature".to_owned()))`
/// and calling `as_str` on it yields `"NewFeature"`.
/// This match expression is forward-compatible when executed with a newer
/// version of SDK where the variant `ResultCode::NewFeature` is defined.
/// Specifically, when `resultcode` represents `NewFeature`,
/// the execution path will hit the second last match arm as before by virtue of
/// calling `as_str` on `ResultCode::NewFeature` also yielding `"NewFeature"`.
///
/// Explicitly matching on the `Unknown` variant should
/// be avoided for two reasons:
/// - The inner data `UnknownVariantValue` is opaque, and no further information can be extracted.
/// - It might inadvertently shadow other intended match arms.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(
    ::std::clone::Clone,
    ::std::cmp::Eq,
    ::std::cmp::Ord,
    ::std::cmp::PartialEq,
    ::std::cmp::PartialOrd,
    ::std::fmt::Debug,
    ::std::hash::Hash,
)]
pub enum ResultCode {
    /// Your input contains inappropriate language
    Inappropriate,
    /// A valid response from LLM
    Llm,
    /// Could not Answer the Question
    NoAnswer,
    /// Out of domain error
    Ood,
    /// Input contains personally identifiable information
    Pii,
    /// Input is similar to prohibited questions
    Prohibited,
    /// `Unknown` contains new variants that have been added since this code was generated.
    #[deprecated(
        note = "Don't directly match on `Unknown`. See the docs on this enum for the correct way to handle unknown variants."
    )]
    Unknown(crate::primitives::sealed_enum_unknown::UnknownVariantValue),
}
impl ::std::convert::From<&str> for ResultCode {
    fn from(s: &str) -> Self {
        match s {
            "INAPPROPRIATE" => ResultCode::Inappropriate,
            "LLM" => ResultCode::Llm,
            "NO_ANSWER" => ResultCode::NoAnswer,
            "OOD" => ResultCode::Ood,
            "PII" => ResultCode::Pii,
            "PROHIBITED" => ResultCode::Prohibited,
            other => ResultCode::Unknown(crate::primitives::sealed_enum_unknown::UnknownVariantValue(
                other.to_owned(),
            )),
        }
    }
}
impl ::std::str::FromStr for ResultCode {
    type Err = ::std::convert::Infallible;

    fn from_str(s: &str) -> ::std::result::Result<Self, <Self as ::std::str::FromStr>::Err> {
        ::std::result::Result::Ok(ResultCode::from(s))
    }
}
impl ResultCode {
    /// Returns the `&str` value of the enum member.
    pub fn as_str(&self) -> &str {
        match self {
            ResultCode::Inappropriate => "INAPPROPRIATE",
            ResultCode::Llm => "LLM",
            ResultCode::NoAnswer => "NO_ANSWER",
            ResultCode::Ood => "OOD",
            ResultCode::Pii => "PII",
            ResultCode::Prohibited => "PROHIBITED",
            ResultCode::Unknown(value) => value.as_str(),
        }
    }

    /// Returns all the `&str` representations of the enum members.
    pub const fn values() -> &'static [&'static str] {
        &["INAPPROPRIATE", "LLM", "NO_ANSWER", "OOD", "PII", "PROHIBITED"]
    }
}
impl ::std::convert::AsRef<str> for ResultCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl ResultCode {
    /// Parses the enum value while disallowing unknown variants.
    ///
    /// Unknown variants will result in an error.
    pub fn try_parse(value: &str) -> ::std::result::Result<Self, crate::error::UnknownVariantError> {
        match Self::from(value) {
            #[allow(deprecated)]
            Self::Unknown(_) => ::std::result::Result::Err(crate::error::UnknownVariantError::new(value)),
            known => Ok(known),
        }
    }
}
impl ::std::fmt::Display for ResultCode {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ResultCode::Inappropriate => write!(f, "INAPPROPRIATE"),
            ResultCode::Llm => write!(f, "LLM"),
            ResultCode::NoAnswer => write!(f, "NO_ANSWER"),
            ResultCode::Ood => write!(f, "OOD"),
            ResultCode::Pii => write!(f, "PII"),
            ResultCode::Prohibited => write!(f, "PROHIBITED"),
            ResultCode::Unknown(value) => write!(f, "{}", value),
        }
    }
}

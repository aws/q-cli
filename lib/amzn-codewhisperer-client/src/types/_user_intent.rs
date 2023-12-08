// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// When writing a match expression against `UserIntent`, it is important to ensure
/// your code is forward-compatible. That is, if a match arm handles a case for a
/// feature that is supported by the service but has not been represented as an enum
/// variant in a current version of SDK, your code should continue to work when you
/// upgrade SDK to a future version in which the enum does include a variant for that
/// feature.
///
/// Here is an example of how you can make a match expression forward-compatible:
///
/// ```text
/// # let userintent = unimplemented!();
/// match userintent {
///     UserIntent::ApplyCommonBestPractices => { /* ... */ },
///     UserIntent::CiteSources => { /* ... */ },
///     UserIntent::ExplainCodeSelection => { /* ... */ },
///     UserIntent::ExplainLineByLine => { /* ... */ },
///     UserIntent::ImproveCode => { /* ... */ },
///     UserIntent::ShowExamples => { /* ... */ },
///     UserIntent::SuggestAlternateImplementation => { /* ... */ },
///     other @ _ if other.as_str() == "NewFeature" => { /* handles a case for `NewFeature` */ },
///     _ => { /* ... */ },
/// }
/// ```
/// The above code demonstrates that when `userintent` represents
/// `NewFeature`, the execution path will lead to the second last match arm,
/// even though the enum does not contain a variant `UserIntent::NewFeature`
/// in the current version of SDK. The reason is that the variable `other`,
/// created by the `@` operator, is bound to
/// `UserIntent::Unknown(UnknownVariantValue("NewFeature".to_owned()))`
/// and calling `as_str` on it yields `"NewFeature"`.
/// This match expression is forward-compatible when executed with a newer
/// version of SDK where the variant `UserIntent::NewFeature` is defined.
/// Specifically, when `userintent` represents `NewFeature`,
/// the execution path will hit the second last match arm as before by virtue of
/// calling `as_str` on `UserIntent::NewFeature` also yielding `"NewFeature"`.
///
/// Explicitly matching on the `Unknown` variant should
/// be avoided for two reasons:
/// - The inner data `UnknownVariantValue` is opaque, and no further information can be extracted.
/// - It might inadvertently shadow other intended match arms.
/// User Intent
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
pub enum UserIntent {
    /// Apply Common Best Practices
    ApplyCommonBestPractices,
    /// Cite Sources
    CiteSources,
    /// Explain Code Selection
    ExplainCodeSelection,
    /// Explain Code Line By Line
    ExplainLineByLine,
    /// Improve Code
    ImproveCode,
    /// Show More Examples
    ShowExamples,
    /// Suggest Alternative Implementation
    SuggestAlternateImplementation,
    /// `Unknown` contains new variants that have been added since this code was generated.
    #[deprecated(
        note = "Don't directly match on `Unknown`. See the docs on this enum for the correct way to handle unknown variants."
    )]
    Unknown(crate::primitives::sealed_enum_unknown::UnknownVariantValue),
}
impl ::std::convert::From<&str> for UserIntent {
    fn from(s: &str) -> Self {
        match s {
            "APPLY_COMMON_BEST_PRACTICES" => UserIntent::ApplyCommonBestPractices,
            "CITE_SOURCES" => UserIntent::CiteSources,
            "EXPLAIN_CODE_SELECTION" => UserIntent::ExplainCodeSelection,
            "EXPLAIN_LINE_BY_LINE" => UserIntent::ExplainLineByLine,
            "IMPROVE_CODE" => UserIntent::ImproveCode,
            "SHOW_EXAMPLES" => UserIntent::ShowExamples,
            "SUGGEST_ALTERNATE_IMPLEMENTATION" => UserIntent::SuggestAlternateImplementation,
            other => UserIntent::Unknown(crate::primitives::sealed_enum_unknown::UnknownVariantValue(
                other.to_owned(),
            )),
        }
    }
}
impl ::std::str::FromStr for UserIntent {
    type Err = ::std::convert::Infallible;

    fn from_str(s: &str) -> ::std::result::Result<Self, <Self as ::std::str::FromStr>::Err> {
        ::std::result::Result::Ok(UserIntent::from(s))
    }
}
impl UserIntent {
    /// Returns the `&str` value of the enum member.
    pub fn as_str(&self) -> &str {
        match self {
            UserIntent::ApplyCommonBestPractices => "APPLY_COMMON_BEST_PRACTICES",
            UserIntent::CiteSources => "CITE_SOURCES",
            UserIntent::ExplainCodeSelection => "EXPLAIN_CODE_SELECTION",
            UserIntent::ExplainLineByLine => "EXPLAIN_LINE_BY_LINE",
            UserIntent::ImproveCode => "IMPROVE_CODE",
            UserIntent::ShowExamples => "SHOW_EXAMPLES",
            UserIntent::SuggestAlternateImplementation => "SUGGEST_ALTERNATE_IMPLEMENTATION",
            UserIntent::Unknown(value) => value.as_str(),
        }
    }

    /// Returns all the `&str` representations of the enum members.
    pub const fn values() -> &'static [&'static str] {
        &[
            "APPLY_COMMON_BEST_PRACTICES",
            "CITE_SOURCES",
            "EXPLAIN_CODE_SELECTION",
            "EXPLAIN_LINE_BY_LINE",
            "IMPROVE_CODE",
            "SHOW_EXAMPLES",
            "SUGGEST_ALTERNATE_IMPLEMENTATION",
        ]
    }
}
impl ::std::convert::AsRef<str> for UserIntent {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl UserIntent {
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

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// When writing a match expression against `Origin`, it is important to ensure
/// your code is forward-compatible. That is, if a match arm handles a case for a
/// feature that is supported by the service but has not been represented as an enum
/// variant in a current version of SDK, your code should continue to work when you
/// upgrade SDK to a future version in which the enum does include a variant for that
/// feature.
///
/// Here is an example of how you can make a match expression forward-compatible:
///
/// ```text
/// # let origin = unimplemented!();
/// match origin {
///     Origin::Chatbot => { /* ... */ },
///     Origin::Console => { /* ... */ },
///     Origin::Documentation => { /* ... */ },
///     Origin::Ide => { /* ... */ },
///     Origin::Marketing => { /* ... */ },
///     Origin::Md => { /* ... */ },
///     Origin::Mobile => { /* ... */ },
///     Origin::SageMaker => { /* ... */ },
///     Origin::ServiceInternal => { /* ... */ },
///     Origin::UnifiedSearch => { /* ... */ },
///     Origin::UnknownValue => { /* ... */ },
///     other @ _ if other.as_str() == "NewFeature" => { /* handles a case for `NewFeature` */ },
///     _ => { /* ... */ },
/// }
/// ```
/// The above code demonstrates that when `origin` represents
/// `NewFeature`, the execution path will lead to the second last match arm,
/// even though the enum does not contain a variant `Origin::NewFeature`
/// in the current version of SDK. The reason is that the variable `other`,
/// created by the `@` operator, is bound to
/// `Origin::Unknown(UnknownVariantValue("NewFeature".to_owned()))`
/// and calling `as_str` on it yields `"NewFeature"`.
/// This match expression is forward-compatible when executed with a newer
/// version of SDK where the variant `Origin::NewFeature` is defined.
/// Specifically, when `origin` represents `NewFeature`,
/// the execution path will hit the second last match arm as before by virtue of
/// calling `as_str` on `Origin::NewFeature` also yielding `"NewFeature"`.
///
/// Explicitly matching on the `Unknown` variant should
/// be avoided for two reasons:
/// - The inner data `UnknownVariantValue` is opaque, and no further information can be extracted.
/// - It might inadvertently shadow other intended match arms.
/// Enum to represent the origin application conversing with Sidekick.
///
/// _Note: `Origin::Unknown` has been renamed to `::UnknownValue`._
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
pub enum Origin {
    /// AWS Chatbot
    Chatbot,
    /// AWS Management Console (https://<region>.console.aws.amazon.com)
    Console,
    /// AWS Documentation Website (https://docs.aws.amazon.com)
    Documentation,
    /// Any IDE caller.
    Ide,
    /// AWS Marketing Website (https://aws.amazon.com)
    Marketing,
    /// MD.
    Md,
    /// AWS Mobile Application (ACMA)
    Mobile,
    /// Amazon SageMaker's Rome Chat.
    SageMaker,
    /// Internal Service Traffic (Integ Tests, Canaries, etc.). This is the default when no Origin
    /// header present in request.
    ServiceInternal,
    /// Unified Search in AWS Management Console (https://<region>.console.aws.amazon.com)
    UnifiedSearch,
    /// Origin header is not set.
    ///
    /// _Note: `::Unknown` has been renamed to `::UnknownValue`._
    UnknownValue,
    /// `Unknown` contains new variants that have been added since this code was generated.
    #[deprecated(
        note = "Don't directly match on `Unknown`. See the docs on this enum for the correct way to handle unknown variants."
    )]
    Unknown(crate::primitives::sealed_enum_unknown::UnknownVariantValue),
}
impl ::std::convert::From<&str> for Origin {
    fn from(s: &str) -> Self {
        match s {
            "CHATBOT" => Origin::Chatbot,
            "CONSOLE" => Origin::Console,
            "DOCUMENTATION" => Origin::Documentation,
            "IDE" => Origin::Ide,
            "MARKETING" => Origin::Marketing,
            "MD" => Origin::Md,
            "MOBILE" => Origin::Mobile,
            "SAGE_MAKER" => Origin::SageMaker,
            "SERVICE_INTERNAL" => Origin::ServiceInternal,
            "UNIFIED_SEARCH" => Origin::UnifiedSearch,
            "UNKNOWN" => Origin::UnknownValue,
            other => Origin::Unknown(crate::primitives::sealed_enum_unknown::UnknownVariantValue(
                other.to_owned(),
            )),
        }
    }
}
impl ::std::str::FromStr for Origin {
    type Err = ::std::convert::Infallible;

    fn from_str(s: &str) -> ::std::result::Result<Self, <Self as ::std::str::FromStr>::Err> {
        ::std::result::Result::Ok(Origin::from(s))
    }
}
impl Origin {
    /// Returns the `&str` value of the enum member.
    pub fn as_str(&self) -> &str {
        match self {
            Origin::Chatbot => "CHATBOT",
            Origin::Console => "CONSOLE",
            Origin::Documentation => "DOCUMENTATION",
            Origin::Ide => "IDE",
            Origin::Marketing => "MARKETING",
            Origin::Md => "MD",
            Origin::Mobile => "MOBILE",
            Origin::SageMaker => "SAGE_MAKER",
            Origin::ServiceInternal => "SERVICE_INTERNAL",
            Origin::UnifiedSearch => "UNIFIED_SEARCH",
            Origin::UnknownValue => "UNKNOWN",
            Origin::Unknown(value) => value.as_str(),
        }
    }

    /// Returns all the `&str` representations of the enum members.
    pub const fn values() -> &'static [&'static str] {
        &[
            "CHATBOT",
            "CONSOLE",
            "DOCUMENTATION",
            "IDE",
            "MARKETING",
            "MD",
            "MOBILE",
            "SAGE_MAKER",
            "SERVICE_INTERNAL",
            "UNIFIED_SEARCH",
            "UNKNOWN",
        ]
    }
}
impl ::std::convert::AsRef<str> for Origin {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl Origin {
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
impl ::std::fmt::Display for Origin {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Origin::Chatbot => write!(f, "CHATBOT"),
            Origin::Console => write!(f, "CONSOLE"),
            Origin::Documentation => write!(f, "DOCUMENTATION"),
            Origin::Ide => write!(f, "IDE"),
            Origin::Marketing => write!(f, "MARKETING"),
            Origin::Md => write!(f, "MD"),
            Origin::Mobile => write!(f, "MOBILE"),
            Origin::SageMaker => write!(f, "SAGE_MAKER"),
            Origin::ServiceInternal => write!(f, "SERVICE_INTERNAL"),
            Origin::UnifiedSearch => write!(f, "UNIFIED_SEARCH"),
            Origin::UnknownValue => write!(f, "UNKNOWN"),
            Origin::Unknown(value) => write!(f, "{}", value),
        }
    }
}

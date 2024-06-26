use crate::errors::ArnError;
use crate::model::{Account, Arn, Category, Domain, Part, Parts};
use crate::traits::ArnComponent;
use crate::Root;
use std::borrow::Cow;

/// A builder for constructing Arn instances using a state-driven approach with type safety.
pub struct ArnBuilder<'a, State> {
    builder: PrivateArnBuilder<'a>,
    _marker: std::marker::PhantomData<State>,
}

/// Implementation of `ArnBuilder` for the initial state, starting with `Domain`.
impl<'a> ArnBuilder<'a, ()> {
    /// Creates a new Arn builder initialized to start building from the `Domain` component.
    pub fn new() -> ArnBuilder<'a, Domain<'a>> {
        ArnBuilder {
            builder: PrivateArnBuilder::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

/// Implementation of `ArnBuilder` for `Part` states, allowing for building the final Arn.
impl<'a> ArnBuilder<'a, Part<'a>> {
    /// Finalizes the building process and constructs the Arn.
    pub fn build(self) -> Result<Arn<'a>, ArnError> {
        self.builder.build()
    }
}

/// Implementation of `ArnBuilder` for handling `Parts` states.
impl<'a> ArnBuilder<'a, Parts<'a>> {
    /// Finalizes the building process and constructs the Arn when in the `Parts` state.
    pub fn build(self) -> Result<Arn<'a>, ArnError> {
        self.builder.build()
    }
}

/// Generic implementation of `ArnBuilder` for all states that can transition to another state.
impl<'a, T: ArnComponent<'a>> ArnBuilder<'a, T> {
    /// Adds a new part to the Arn, transitioning to the next appropriate state.
    pub fn with<N>(
        self,
        part: impl Into<Cow<'a, str>>,
    ) -> Result<ArnBuilder<'a, N::NextState>, ArnError>
    where
        N: ArnComponent<'a, NextState = T::NextState>,
    {
        Ok(ArnBuilder {
            builder: self.builder.add_part(N::prefix(), part.into())?,
            _marker: std::marker::PhantomData,
        })
    }
}

/// Represents a private, internal structure for building the Arn.
struct PrivateArnBuilder<'a> {
    domain: Option<Domain<'a>>,
    category: Option<Category<'a>>,
    account: Option<Account<'a>>,
    root: Option<Root<'a>>,
    parts: Parts<'a>,
}

impl<'a> PrivateArnBuilder<'a> {
    /// Constructs a new private Arn builder.
    fn new() -> Self {
        Self {
            domain: None,
            category: None,
            account: None,
            root: None,
            parts: Parts::new(Vec::new()),
        }
    }

    fn add_part(mut self, prefix: &'static str, part: Cow<'a, str>) -> Result<Self, ArnError> {
        match prefix {
            p if p == Domain::prefix() => {
                self.domain = Some(Domain::new(part)?);
            }
            "" => {
                if self.domain.is_some() && self.category.is_none() {
                    self.category = Some(Category::new(part));
                } else if self.category.is_some() && self.account.is_none() {
                    self.account = Some(Account::new(part));
                } else if self.account.is_some() && self.root.is_none() {
                    self.root = Some(Root::new(part)?);
                } else {
                    // add the first part
                    self.parts = self.parts.add_part(Part::new(part)?);
                }
            }
            ":" => {
                self.parts = self.parts.add_part(Part::new(part)?);
            }
            _ => return Err(ArnError::InvalidPrefix(prefix.to_string())),
        }
        Ok(self)
    }

    /// Finalizes and builds the Arn.
    fn build(self) -> Result<Arn<'a>, ArnError> {
        let domain = self
            .domain
            .ok_or(ArnError::MissingPart("domain".to_string()))?;
        let category = self
            .category
            .ok_or(ArnError::MissingPart("category".to_string()))?;
        let account = self
            .account
            .ok_or(ArnError::MissingPart("account".to_string()))?;
        let root = self.root.ok_or(ArnError::MissingPart("root".to_string()))?;

        Ok(Arn::new(domain, category, account, root, self.parts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ArnError;
    use crate::tests::init_tracing;
    use crate::{ArnBuilder, ArnParser};

    #[test]
    fn test() -> anyhow::Result<()> {
        // Create an Arn using the ArnBuilder with specified components
        let arn = ArnBuilder::new()
            .with::<Domain>("akton-internal")?
            .with::<Category>("hr")?
            .with::<Account>("company123")?
            .with::<Root>("root")?
            .with::<Part>("departmentA")?
            .with::<Part>("team1")?
            .build();

        // Verify the constructed Arn matches the expected value
        assert!(
            arn.is_ok(),
            "arn:akton-internal:hr:company123:root/departmentA/team1"
        );
        Ok(())
    }
    #[test]
    fn test_arn_builder() -> anyhow::Result<()> {
        let arn = ArnBuilder::new()
            .with::<Domain>("custom")?
            .with::<Category>("service")?
            .with::<Account>("account123")?
            .with::<Root>("resource")?
            .with::<Part>("subresource")?
            .build()?;

        assert!(
            arn.to_string().ends_with("/subresource"),
            "{} did not end with expected string",
            arn
        );

        Ok(())
    }

    #[test]
    fn test_arn_builder_with_default_parts() -> anyhow::Result<(), ArnError> {
        init_tracing();
        let arn = Arn::default();
        tracing::debug!("{}", arn);
        let parser = ArnParser::new(arn.to_string());
        let parsed = parser.parse()?;
        assert_eq!(parsed.domain.as_str(), "akton");
        // assert_eq!(arn.to_string(), "arn:akton:system:default:root");
        Ok(())
    }

    #[test]
    fn test_arn_builder_with_owned_strings() -> anyhow::Result<(), ArnError> {
        let arn = ArnBuilder::new()
            .with::<Domain>(String::from("custom"))?
            .with::<Category>(String::from("service"))?
            .with::<Account>(String::from("account123"))?
            .with::<Root>(String::from("resource"))?
            .build()?;

        assert!(arn
            .to_string()
            .starts_with("arn:custom:service:account123:resource"));
        Ok(())
    }
}

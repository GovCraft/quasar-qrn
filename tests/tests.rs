use akton_arn::*;

/// Tests for the Akton ARN implementation

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
fn test_parser() -> anyhow::Result<()> {
    // Create an ArnParser with a specific Arn string
    let parser = ArnParser::new("arn:akton-internal:hr:company123:root/departmentA/team1");

    // Parse the Arn string into its components
    let result = parser.parse();

    // Verify the parser returns a successful result
    assert!(
        result.is_ok(),
        "Parser should return Ok, but returned Err with message: {:?}",
        result.err()
    );

    // Extract the components from the result
    let arn = result.unwrap();

    // Verify each component matches the expected value
    assert_eq!(
        arn.domain.to_string(),
        "akton-internal",
        "Domain should be 'akton-internal'"
    );
    assert_eq!(arn.category.to_string(), "hr", "Category should be 'hr'");
    assert_eq!(
        arn.account.to_string(),
        "company123",
        "Account should be 'company123'"
    );
    assert_eq!(
        arn.parts.to_string(),
        "departmentA/team1",
        "Parts should match expected values"
    );
    Ok(())
}

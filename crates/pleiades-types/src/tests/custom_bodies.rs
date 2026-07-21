use crate::*;

#[test]
fn custom_body_id_validate_rejects_blank_padding_and_separators() {
    assert_eq!(
        CustomBodyId::new("", "433-Eros")
            .validate()
            .expect_err("blank catalogs should be rejected")
            .to_string(),
        "custom body id catalog must not be blank"
    );

    assert_eq!(
        CustomBodyId::new("asteroid", " 433-Eros ")
            .validate()
            .expect_err("padded designations should be rejected")
            .to_string(),
        "custom body id designation must not have leading or trailing whitespace"
    );

    assert_eq!(
        CustomBodyId::new("asteroid:catalog", "433-Eros")
            .validate()
            .expect_err("separator characters should be rejected")
            .to_string(),
        "custom body id catalog must not contain ':'"
    );
}

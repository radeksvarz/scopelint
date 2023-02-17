use crate::check::utils::{offset_to_line, FileKind, InvalidItem, IsFileKind, Name, Validator};
use once_cell::sync::Lazy;
use regex::Regex;
use solang_parser::pt::{ContractPart, FunctionDefinition, SourceUnit, SourceUnitPart};
use std::{error::Error, path::Path};

// A regex matching valid test names, see the `validate_test_names_regex` test for examples.
static RE_VALID_TEST_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^test(Fork)?(Fuzz)?(_Revert(If|When|On))?_(\w+)*$").unwrap());

pub fn validate(
    file: &Path,
    content: &str,
    pt: &SourceUnit,
) -> Result<Vec<InvalidItem>, Box<dyn Error>> {
    if !file.is_file_kind(FileKind::TestContracts) {
        return Ok(Vec::new())
    }

    let mut invalid_items: Vec<InvalidItem> = Vec::new();
    for element in &pt.0 {
        match element {
            SourceUnitPart::FunctionDefinition(f) => {
                if let Some(invalid_item) = validate_name(file, content, f) {
                    invalid_items.push(invalid_item);
                }
            }
            SourceUnitPart::ContractDefinition(c) => {
                for el in &c.parts {
                    if let ContractPart::FunctionDefinition(f) = el {
                        if let Some(invalid_item) = validate_name(file, content, f) {
                            invalid_items.push(invalid_item);
                        }
                    }
                }
            }
            _ => (),
        }
    }
    Ok(invalid_items)
}

fn is_valid_test_name(name: &str) -> bool {
    if !name.starts_with("test") {
        return true // Not a test function, so return true and skip this check.
    }
    name.starts_with("test") && RE_VALID_TEST_NAME.is_match(name)
}

fn validate_name(file: &Path, content: &str, f: &FunctionDefinition) -> Option<InvalidItem> {
    let name = f.name();
    if !is_valid_test_name(&name) {
        Some(InvalidItem::new(
            Validator::Test,
            file.display().to_string(),
            name,
            offset_to_line(content, f.loc.start()),
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_test_names_regex() {
        let allowed_names = vec![
            "test_Description",
            "test_Increment",
            "testFuzz_Description",
            "testFork_Description",
            "testForkFuzz_Description",
            "testForkFuzz_Description_MoreInfo",
            "test_RevertIf_Condition",
            "test_RevertWhen_Condition",
            "test_RevertOn_Condition",
            "test_RevertOn_Condition_MoreInfo",
            "testFuzz_RevertIf_Condition",
            "testFuzz_RevertWhen_Condition",
            "testFuzz_RevertOn_Condition",
            "testFuzz_RevertOn_Condition_MoreInfo",
            "testForkFuzz_RevertIf_Condition",
            "testForkFuzz_RevertWhen_Condition",
            "testForkFuzz_RevertOn_Condition",
            "testForkFuzz_RevertOn_Condition_MoreInfo",
            "testForkFuzz_RevertOn_Condition_MoreInfo_Wow",
            "testForkFuzz_RevertOn_Condition_MoreInfo_Wow_As_Many_Underscores_As_You_Want",
        ];

        let disallowed_names = [
            "test",
            "testDescription",
            "testDescriptionMoreInfo",
            // TODO The below are tough to prevent without regex look-ahead support.
            // "test_RevertIfCondition",
            // "test_RevertWhenCondition",
            // "test_RevertOnCondition",
            // "testFuzz_RevertIfDescription",
            // "testFuzz_RevertWhenDescription",
            // "testFuzz_RevertOnDescription",
            // "testForkFuzz_RevertIfCondition",
            // "testForkFuzz_RevertWhenCondition",
            // "testForkFuzz_RevertOnCondition",
        ];

        for name in allowed_names {
            assert_eq!(is_valid_test_name(name), true, "{name}");
        }

        for name in disallowed_names {
            assert_eq!(is_valid_test_name(name), false, "{name}");
        }
    }
}

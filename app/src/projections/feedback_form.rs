use ferro::{ActionDef, DataType, FieldMeaning, InputDef, ServiceDef};

/// Build the Feedback Form service projection.
///
/// Models a data collection form with write-only submission fields.
/// Designed to derive the Collect intent via >50% writable fields,
/// write_only markers, and an action with multiple inputs.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("feedback_form")
        .display_name("Feedback Form")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("subject", DataType::String, FieldMeaning::EntityName)
        .field("rating", DataType::Integer, FieldMeaning::Quantity)
        .write_only_field("comment", DataType::String, FieldMeaning::FreeText)
        .write_only_field("contact_email", DataType::String, FieldMeaning::Email)
        .field("category", DataType::String, FieldMeaning::Category)
        .action(
            ActionDef::new("submit_feedback")
                .display_name("Submit Feedback")
                .input(InputDef::new(
                    "subject",
                    DataType::String,
                    FieldMeaning::EntityName,
                ))
                .input(InputDef::new(
                    "rating",
                    DataType::Integer,
                    FieldMeaning::Quantity,
                ))
                .input(InputDef::new(
                    "comment",
                    DataType::String,
                    FieldMeaning::FreeText,
                )),
        )
}

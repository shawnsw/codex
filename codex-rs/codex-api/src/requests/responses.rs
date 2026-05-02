use codex_protocol::models::ResponseItem;
use serde_json::Value;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Compression {
    #[default]
    None,
    Zstd,
}

pub(crate) fn attach_item_ids(payload_json: &mut Value, original_items: &[ResponseItem]) {
    let Some(input_value) = payload_json.get_mut("input") else {
        return;
    };
    let Value::Array(items) = input_value else {
        return;
    };

    for (value, item) in items.iter_mut().zip(original_items.iter()) {
        if let Some(id) = response_item_wire_id(item)
            && !id.is_empty()
        {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("id".to_string(), Value::String(id.to_string()));
            }
        }
    }
}

fn response_item_wire_id(item: &ResponseItem) -> Option<&str> {
    match item {
        ResponseItem::Message { id: Some(id), .. } if id.starts_with("msg") => Some(id),
        ResponseItem::Reasoning { id, .. } => Some(id),
        ResponseItem::WebSearchCall { id: Some(id), .. }
        | ResponseItem::FunctionCall { id: Some(id), .. }
        | ResponseItem::ToolSearchCall { id: Some(id), .. }
        | ResponseItem::LocalShellCall { id: Some(id), .. }
        | ResponseItem::CustomToolCall { id: Some(id), .. } => Some(id),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::models::ContentItem;
    use pretty_assertions::assert_eq;

    fn message_with_id(id: &str) -> ResponseItem {
        ResponseItem::Message {
            id: Some(id.to_string()),
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "retry after hook".to_string(),
            }],
            phase: None,
        }
    }

    fn payload_without_ids() -> Value {
        serde_json::json!({
            "input": [
                {
                    "type": "message",
                    "role": "user",
                    "content": [
                        {
                            "type": "input_text",
                            "text": "retry after hook"
                        }
                    ]
                }
            ]
        })
    }

    #[test]
    fn attach_item_ids_skips_local_message_ids() {
        let original_items = vec![message_with_id("c047686e-8eaf-4c21-9eba-e25d3122d898")];
        let mut payload = payload_without_ids();

        attach_item_ids(&mut payload, &original_items);

        assert_eq!(payload["input"][0].get("id"), None);
    }

    #[test]
    fn attach_item_ids_preserves_server_message_ids() {
        let original_items = vec![message_with_id("msg_123")];
        let mut payload = payload_without_ids();

        attach_item_ids(&mut payload, &original_items);

        assert_eq!(
            payload["input"][0].get("id").and_then(Value::as_str),
            Some("msg_123")
        );
    }
}

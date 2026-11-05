use dioxus::prelude::*;
use dioxus_icons::lucide::Check;
use dioxus_primitives::checkbox::{self, CheckboxProps};

static CSS: &str = include_str!("style.css");

#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    rsx! {
        document::Style { {CSS} }
        checkbox::Checkbox {
            class: "dx-checkbox",
            checked: props.checked,
            default_checked: props.default_checked,
            required: props.required,
            disabled: props.disabled,
            name: props.name,
            value: props.value,
            on_checked_change: props.on_checked_change,
            attributes: props.attributes,
            checkbox::CheckboxIndicator { class: "dx-checkbox-indicator",
                Check { size: "1rem" }
            }
        }
    }
}

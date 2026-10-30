use super::*;

#[test]
fn each_row_is_a_label_and_its_value_under_the_state() {
    let rows = [
        Row {
            label: "State",
            value: "on".to_owned(),
        },
        Row {
            label: "Last handshake",
            value: "12 s ago".to_owned(),
        },
    ];

    let view = view(&rows);

    assert_eq!(view.title, "Proxy: on");
    let ViewContent::List { items, .. } = view.content else {
        panic!("a list View");
    };
    let shown: Vec<(&str, &str)> = items
        .iter()
        .map(|item| (item.title.as_str(), item.subtitle.as_deref().unwrap_or("")))
        .collect();
    assert_eq!(shown, [("State", "on"), ("Last handshake", "12 s ago")]);
}

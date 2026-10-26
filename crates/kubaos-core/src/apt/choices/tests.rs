use super::Choices;
use super::Switch;

#[test]
fn choices_round_trip_sorted_under_their_header() {
    let mut choices = Choices::default();
    choices.set("piv", Switch::On);
    choices.set("docker", Switch::Off);

    let text = choices.to_text();

    assert!(text.starts_with("# "));
    assert!(text.ends_with("docker=off\npiv=on\n"));
    assert_eq!(Choices::parse(&text), choices);
}

#[test]
fn a_line_that_isnt_a_choice_is_skipped() {
    let choices = Choices::parse("docker=off\n# u2f=off\nchrome=maybe\nnonsense\n piv = on \n");

    assert_eq!(choices.get("docker"), Some(Switch::Off));
    assert_eq!(choices.get("piv"), Some(Switch::On));
    assert_eq!(choices.get("u2f"), None);
    assert_eq!(choices.get("chrome"), None);
}

#[test]
fn a_later_choice_replaces_an_earlier_one() {
    let mut choices = Choices::parse("docker=off\n");

    choices.set("docker", Switch::On);

    assert_eq!(choices.get("docker"), Some(Switch::On));
}

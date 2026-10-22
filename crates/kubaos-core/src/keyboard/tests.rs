use super::*;

fn l(spelled: &str) -> Layout {
    Layout::parse(spelled)
}

fn known() -> Vec<String> {
    ["us", "it", "de", "pl"].map(String::from).to_vec()
}

#[test]
fn a_layout_reads_and_spells_with_its_variant() {
    assert_eq!(l("us(intl)"), Layout::new("us", "intl"));
    assert_eq!(l(" it "), Layout::new("it", ""));
    assert_eq!(Layout::new("us", "intl").to_string(), "us(intl)");
    assert_eq!(Layout::new("it", "").to_string(), "it");
}

#[test]
fn the_xkb_fields_pair_layouts_with_variants() {
    let layouts = vec![l("us"), l("it(intl)")];

    assert_eq!(to_fields(&layouts), ("us,it".into(), ",intl".into()));
    assert_eq!(from_fields("us,it", ",intl"), layouts);
    assert_eq!(from_fields("us,it", ""), vec![l("us"), l("it")]);
    assert_eq!(from_fields("", ""), Vec::<Layout>::new());
}

#[test]
fn a_comma_separated_list_reads_both_ways() {
    let layouts = parse_list("us, it(intl),");

    assert_eq!(layouts, vec![l("us"), l("it(intl)")]);
    assert_eq!(spell_list(&layouts), "us,it(intl)");
}

#[test]
fn the_file_holds_every_layout_the_first_the_default() {
    let text = default_keyboard(&[l("us"), l("it(intl)")]);

    assert!(
        text.contains("XKBLAYOUT=\"us,it\"\nXKBVARIANT=\",intl\"\n"),
        "{text}"
    );
    assert_eq!(
        KeyboardFile::parse(&text).layouts,
        vec![l("us"), l("it(intl)")]
    );
}

#[test]
fn one_layout_writes_the_file_it_always_did() {
    assert!(default_keyboard(&[l("it")]).contains("XKBLAYOUT=\"it\"\nXKBVARIANT=\"\"\n"));
}

#[test]
fn the_file_reads_in_any_shell_quoting() {
    let f = KeyboardFile::parse(
        "XKBMODEL=pc105\nXKBLAYOUT='de,us'\n  XKBVARIANT=\"nodeadkeys,\"\nXKBOPTIONS=\"grp:alt_shift_toggle\"\n",
    );

    assert_eq!(f.model, "pc105");
    assert_eq!(f.layouts, vec![l("de(nodeadkeys)"), l("us")]);
    assert_eq!(f.options, "grp:alt_shift_toggle");
}

#[test]
fn a_list_has_one_to_four_known_layouts_each_once() {
    assert_eq!(problems(&[l("us"), l("it")], &known()), vec![]);
    assert_eq!(problems(&[], &known()), vec![LayoutsError::Empty]);
    assert_eq!(
        problems(
            &[l("us"), l("it"), l("de"), l("pl"), l("us(intl)")],
            &known()
        ),
        vec![LayoutsError::TooMany]
    );
    assert_eq!(
        problems(&[l("us"), l("us")], &known()),
        vec![LayoutsError::Twice("us".into())]
    );
    assert_eq!(
        problems(&[l("xx")], &known()),
        vec![LayoutsError::Unknown("xx".into())]
    );
    assert_eq!(
        problems(&[l("u\"s")], &known()),
        vec![LayoutsError::Odd("u\"s".into())]
    );
    assert_eq!(
        problems(&[l("xx")], &[]),
        vec![],
        "no list: any name passes"
    );
}

#[test]
fn a_layout_with_two_variants_counts_as_two() {
    assert_eq!(problems(&[l("us"), l("us(intl)")], &known()), vec![]);
}

#[test]
fn changes_add_remove_and_reorder() {
    let now = vec![l("us"), l("it")];

    assert_eq!(
        Change::Add(l("de")).apply(&now, &known()),
        Ok(vec![l("us"), l("it"), l("de")])
    );
    assert_eq!(
        Change::Remove(l("us")).apply(&now, &known()),
        Ok(vec![l("it")])
    );
    assert_eq!(
        Change::MakeDefault(l("it")).apply(&now, &known()),
        Ok(vec![l("it"), l("us")])
    );
    assert_eq!(
        Change::Set(vec![l("pl")]).apply(&now, &known()),
        Ok(vec![l("pl")])
    );
}

#[test]
fn a_change_that_breaks_the_rules_is_refused() {
    let now = vec![l("us"), l("it")];

    assert_eq!(
        Change::Add(l("it")).apply(&now, &known()),
        Err(LayoutsError::Twice("it".into()))
    );
    assert_eq!(
        Change::Remove(l("us")).apply(&[l("us")], &known()),
        Err(LayoutsError::Last)
    );
    assert_eq!(
        Change::Remove(l("de")).apply(&now, &known()),
        Err(LayoutsError::NotThere("de".into()))
    );
    assert_eq!(
        Change::MakeDefault(l("de")).apply(&now, &known()),
        Err(LayoutsError::NotThere("de".into()))
    );
    assert_eq!(
        Change::Set(vec![]).apply(&now, &known()),
        Err(LayoutsError::Empty)
    );
}

#[test]
fn names_come_from_the_rules_list() {
    let lst = "! model\n  pc105           Generic 105-key PC\n\n! layout\n  us              English (US)\n  it              Italian\n\n! variant\n  intl            us: English (US, intl., with dead keys)\n\n! option\n  grp                  Switching to another layout\n";
    let names = names(lst);

    assert_eq!(name(&l("it"), &names), "Italian");
    assert_eq!(
        name(&l("us(intl)"), &names),
        "English (US, intl., with dead keys)"
    );
    assert_eq!(name(&l("xx"), &names), "XX");
    assert!(!names.contains_key("pc105") && !names.contains_key("grp"));
}

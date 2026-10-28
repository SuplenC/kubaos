use super::Page;

#[test]
fn every_page_is_found_by_its_slug() {
    for page in Page::ALL {
        assert_eq!(Page::from_slug(page.slug()), Some(page));
    }
}

#[test]
fn a_name_that_is_no_pages_slug_finds_none() {
    assert_eq!(Page::from_slug("Type"), None);
    assert_eq!(Page::from_slug(""), None);
}

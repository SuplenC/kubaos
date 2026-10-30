//! What a sidebar page holds, as the app describes it: Home, then the sections, each with its
//! sub-sections and the wells of rows under each.

use crate::icon::Icon;

/// Where a row is: its section, its sub-section, the well it sits in, and its place in the well.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RowAt {
    pub section: usize,
    pub sub: usize,
    pub well: usize,
    pub row: usize,
}

/// Every page of a sidebar page: Home, then the sections in the order the sidebar lists them.
#[derive(Debug, Clone)]
pub struct Outline {
    pub home: HomeOutline,
    pub sections: Vec<SectionOutline>,
}

/// Home: the sidebar's first row, and the page that is only search.
#[derive(Debug, Clone)]
pub struct HomeOutline {
    /// The sidebar row's words.
    pub label: String,
    pub icon: Icon,
    /// The page's header: its Japanese accent, title and lede.
    pub accent: String,
    pub title: String,
    pub lede: String,
    /// The search field's placeholder.
    pub placeholder: String,
}

/// A section: its sidebar row and its page.
#[derive(Debug, Clone)]
pub struct SectionOutline {
    pub title: String,
    pub icon: Icon,
    /// The page's Japanese accent and lede, under and over its title.
    pub accent: String,
    pub lede: String,
    pub subs: Vec<SubOutline>,
}

/// A sub-section: a heading on the page, a row in the sidebar under its open section, and the
/// wells of rows under the heading.
#[derive(Debug, Clone)]
pub struct SubOutline {
    pub title: String,
    /// How many rows each well holds, in the order they show.
    pub wells: Vec<usize>,
}

impl SectionOutline {
    #[must_use]
    pub fn new(
        icon: Icon,
        title: impl Into<String>,
        accent: impl Into<String>,
        lede: impl Into<String>,
    ) -> SectionOutline {
        SectionOutline {
            title: title.into(),
            icon,
            accent: accent.into(),
            lede: lede.into(),
            subs: Vec::new(),
        }
    }

    /// `sub` after the sub-sections before it.
    #[must_use]
    pub fn with_sub(mut self, sub: SubOutline) -> Self {
        self.subs.push(sub);
        self
    }
}

impl SubOutline {
    #[must_use]
    pub fn new(title: impl Into<String>) -> SubOutline {
        SubOutline {
            title: title.into(),
            wells: Vec::new(),
        }
    }

    /// A well of `rows` rows after the wells before it.
    #[must_use]
    pub fn well(mut self, rows: usize) -> Self {
        self.wells.push(rows);
        self
    }
}

impl Outline {
    /// Every row of `section`, in reading order: by sub-section, then well, then row.
    pub fn rows(&self, section: usize) -> impl Iterator<Item = RowAt> + '_ {
        self.sections.get(section).into_iter().flat_map(move |outline| {
            outline.subs.iter().enumerate().flat_map(move |(sub, shape)| {
                shape.wells.iter().enumerate().flat_map(move |(well, &rows)| {
                    (0..rows).map(move |row| RowAt {
                        section,
                        sub,
                        well,
                        row,
                    })
                })
            })
        })
    }

    /// How many rows `section` holds.
    pub fn row_count(&self, section: usize) -> usize {
        self.sections
            .get(section)
            .map_or(0, |s| s.subs.iter().flat_map(|sub| &sub.wells).sum())
    }

    /// The row `index` places into `section`'s reading order.
    pub fn row_at(&self, section: usize, index: usize) -> Option<RowAt> {
        self.rows(section).nth(index)
    }

    /// Where `at` falls in its section's reading order.
    pub fn index_of(&self, at: RowAt) -> Option<usize> {
        self.rows(at.section).position(|row| row == at)
    }

    /// The first row of `sub` in `section`, as a place in the section's reading order.
    pub fn first_of_sub(&self, section: usize, sub: usize) -> Option<usize> {
        self.rows(section).position(|row| row.sub == sub)
    }

    /// How many sub-sections `section` has.
    pub fn sub_count(&self, section: usize) -> usize {
        self.sections.get(section).map_or(0, |s| s.subs.len())
    }
}

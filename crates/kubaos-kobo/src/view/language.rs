//! The Language screen: the language list and the keyboard layouts that go with it.

use kubaos_ui::Look;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::field::labelled;
use kubaos_ui::component::list::list_row;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::flow;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::header;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::choices;
use super::input;
use crate::app::App;
use crate::app::F;
use crate::app::List;
use crate::app::Message;
use crate::app::ids;
use crate::data;
use crate::theme::jp;
use crate::ui;
use crate::ui::El;

/// The keymap list: every layout, a check mark on the picked ones, the highlight on the row Space
/// acts on.
fn keymap_rows<'a>(app: &'a App, look: &Look) -> Vec<El<'a>> {
    let at = app.keymap_row();
    let default = app.form.keymaps.first();
    app.list_items(List::Keymap)
        .into_iter()
        .map(|v| {
            let here = v == at;
            let mut row = list_row(data::keymap_name(&v).into_owned())
                .detail(v.clone())
                .selected(here)
                .checked(app.form.keymap_picked(&v));
            if default == Some(&v) && app.form.keymaps.len() > 1 {
                row = row.chip(ChipKind::Accent, "Default");
            }
            row.on_press(Message::Pick(List::Keymap, v)).view(look)
        })
        .collect()
}

/// The picked layouts, the default first. A click makes one the default; its close button unpicks
/// it, all but the last one.
fn keymap_chips<'a>(app: &'a App, look: &Look) -> El<'a> {
    let keymaps = &app.form.keymaps;
    let chips = keymaps.iter().enumerate().map(|(i, k)| {
        let lead = i == 0;
        chip(
            if lead { ChipKind::Selected } else { ChipKind::Neutral },
            data::keymap_name(k).into_owned(),
        )
        .icon_maybe(lead.then_some(Icon::Check))
        .on_press_maybe((!lead).then(|| Message::KeymapDefault(k.clone())))
        .on_remove_maybe((keymaps.len() > 1).then(|| Message::KeymapDrop(k.clone())))
        .view(look)
    });
    let hint = if keymaps.len() > 1 {
        text(
            "The first is the default. Click one to make it the default.",
            Type::Caption,
            look.colors.on_surface_variant,
        )
    } else {
        ui::hint_row(
            look,
            &[("Space", "picks more"), ("Shift+Space", "makes one the default")],
        )
    };
    stack(look, Space::S8)
        .push(flow(look, Space::S8, chips))
        .push(hint)
        .into()
}

pub(super) fn language<'a>(app: &'a App, look: &Look) -> El<'a> {
    let rows = |list: List, cur: &str| -> Vec<El<'a>> {
        app.list_items(list)
            .into_iter()
            .map(|v| {
                let sel = v == cur;
                list_row(data::locale_name(&v).into_owned())
                    .detail(v.clone())
                    .selected(sel)
                    .checked(sel)
                    .on_press(Message::Pick(list, v))
                    .view(look)
            })
            .collect()
    };
    let languages = stack(look, Space::S8)
        .height(Extent::Fill)
        .push(labelled(
            look,
            "Language",
            None,
            input(app, F::LocaleQ, "Search languages", Some(Icon::Search), look),
            None,
        ))
        .push(choices(
            ids::LOCALE_LIST,
            rows(List::Locale, &app.form.locale),
            app.is_focused(F::LocaleQ),
            look,
        ));
    let keyboards = stack(look, Space::S8)
        .height(Extent::Fill)
        .push(labelled(
            look,
            "Keyboard",
            None,
            input(app, F::KeymapQ, "Search layouts", Some(Icon::Search), look),
            None,
        ))
        .push(choices(
            ids::KEYMAP_LIST,
            keymap_rows(app, look),
            app.is_focused(F::KeymapQ),
            look,
        ))
        .push(keymap_chips(app, look))
        .push(input(
            app,
            F::KeyboardTest,
            "Mash some keys to check the layout",
            Some(Icon::Keyboard),
            look,
        ));
    stack(look, Space::S24)
        .height(Extent::Fill)
        .push(header(
            look,
            jp::LANGUAGE,
            "Language and keyboard",
            "The language you want to read, and the layouts your fingers actually know. Type to search, arrows \
             to move, Space to pick a layout.",
        ))
        .push(grid(look, 2, Space::S20).fill_height().push(languages).push(keyboards))
        .into()
}

//! The Time zone screen: the searchable list of zones.

use iced::Length;
use iced::widget;
use kubaos_ui::Look;
use kubaos_ui::component::card::card_sized;
use kubaos_ui::component::list::list_row;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::gap;
use kubaos_ui::layout::header;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
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
use crate::ui::El;

pub(super) fn timezone<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let rows: Vec<El> = app
        .list_items(List::Timezone)
        .into_iter()
        .map(|tz| {
            let sel = tz == app.form.timezone;
            let (city, region) = data::split_tz(&tz);
            list_row(city)
                .detail(region)
                .selected(sel)
                .checked(sel)
                .on_press(Message::Pick(List::Timezone, tz))
                .view(look)
        })
        .collect();
    let (city, region) = data::split_tz(&app.form.timezone);
    let side = card_sized(
        look,
        stack(look, Gap::NONE)
            .height(Extent::Fill)
            .push(icon(look, Icon::Public, IconSize::S48, c.primary))
            .push(gap(Space::S16))
            .push(text(city, Type::Headline, c.on_surface))
            .push(text(region, Type::BodyLarge, c.on_surface_variant))
            .push(widget::Space::new().height(Length::Fill))
            .push(text(
                "Your clock, calendar and every timestamp in your logs follow this.",
                Type::Small,
                c.on_surface_variant,
            ))
            .push(gap(Space::S8))
            .push(text(app.form.timezone.clone(), Type::Mono, c.on_surface_variant)),
        Extent::Fill,
    );
    let search = stack(look, Space::S8)
        .height(Extent::Fill)
        .push(input(app, F::TzQ, "Search cities or regions", Some(Icon::Search), look))
        .push(choices(ids::TZ_LIST, rows, app.is_focused(F::TzQ), look));
    stack(look, Space::S24)
        .height(Extent::Fill)
        .push(header(
            look,
            jp::TIMEZONE,
            "Time zone",
            "Where your clock lives. Start typing a city.",
        ))
        .push(split(look, &[2, 1], Space::S20).fill_height().push(search).push(side))
        .into()
}

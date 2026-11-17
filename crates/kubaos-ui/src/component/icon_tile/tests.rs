use std::borrow::Cow;

use iced::Element;
use iced::Pixels;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::advanced::graphics::text::font_system;
use iced::advanced::renderer::Headless;
use iced::widget;
use iced::widget::image;
use iced_runtime::UserInterface;
use iced_runtime::user_interface;
use iced_test::selector;
use iced_test::simulator::Simulator;
use kubaos_geometry::Node;
use kubaos_geometry::Role;

use super::TileSize;
use super::icon_tile;
use super::initial::initial;
use super::initial_tile;
use crate::icon::Icon;
use crate::look::Look;
use crate::probe::Collect;
use crate::scale::Scale;
use crate::tokens;
use crate::tokens::Mode;

fn settings() -> iced::Settings {
    iced::Settings {
        fonts: crate::text::FONTS.iter().map(|f| (*f).into()).collect(),
        default_font: crate::text::default_font(),
        ..iced::Settings::default()
    }
}

fn bounds(element: Element<'_, ()>) -> Rectangle {
    let probe = widget::container(element).id("probe");
    let mut ui = Simulator::with_size(settings(), Size::new(400.0, 400.0), probe);
    ui.find(selector::id("probe")).expect("probe").bounds()
}

fn picture() -> image::Handle {
    image::Handle::from_rgba(4, 4, vec![255; 4 * 4 * 4])
}

#[test]
fn a_tile_is_its_size_square_whatever_it_shows() {
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);
        for size in TileSize::ALL {
            let side = f32::from(size.side().px());

            let symbol = bounds(icon_tile(Icon::Notifications, size).view(&look));
            let app = bounds(
                icon_tile(Icon::Notifications, size)
                    .picture(Some(picture()))
                    .view(&look),
            );
            let photo = bounds(icon_tile(Icon::Notifications, size).photo(Some(picture())).view(&look));

            assert!(
                (symbol.width - side).abs() < 1e-4 && (symbol.height - side).abs() < 1e-4,
                "{scale:?} {size:?}: {symbol:?}"
            );
            assert_eq!(symbol, app, "{scale:?} {size:?}");
            assert_eq!(symbol, photo, "{scale:?} {size:?}");
        }
    }
}

/// `view`'s geometry tree, laid out headless.
fn tree(view: Element<'_, ()>) -> Node {
    {
        let mut system = font_system().write().expect("font system lock poisoned");
        for font in crate::text::FONTS {
            system.load_font(Cow::Borrowed(*font));
        }
    }
    let mut renderer = iced::futures::executor::block_on(<Renderer as Headless>::new(
        crate::text::default_font(),
        Pixels(f32::from(tokens::Type::Body.size())),
        Some("tiny-skia"),
    ))
    .expect("the tiny-skia renderer");
    let mut ui = UserInterface::build(
        view,
        Size::new(400.0, 400.0),
        user_interface::Cache::default(),
        &mut renderer,
    );
    let mut collect = Collect::new();
    ui.operate(&renderer, &mut collect);
    collect.into_tree().expect("the view reports boxes")
}

/// The first text in `node`, and the node it is in.
fn letter(node: &Node) -> Option<&Node> {
    if matches!(node.role, Role::Text { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(letter)
}

#[test]
fn an_initial_is_the_first_character_of_the_name_upper_cased() {
    let cases = [
        ("Blender", "B"),
        ("  k9s", "K"),
        ("日本語入力", "日"),
        ("e\u{301}clair", "E\u{301}"),
        ("𝒜lpha", "𝒜"),
        ("ßeta", "SS"),
        ("", ""),
    ];

    for (name, expected) in cases {
        assert_eq!(initial(name), expected, "{name}");
    }
}

#[test]
fn an_initial_tile_says_it_shows_initials() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let tile = tree(initial_tile("Blender", TileSize::Md).view(&look));

    assert!(
        matches!(&tile.role, Role::Component { name, variant, .. } if name == "IconTile" && variant == "Md initials"),
        "{:?}",
        tile.role
    );
}

#[test]
fn an_initial_sits_centred_on_whole_device_pixels_its_line_tall() {
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);
        for size in TileSize::ALL {
            for name in ["Blender", "LM Studio", "k9s", "日本語入力"] {
                let tile = tree(initial_tile(name, size).view(&look));
                let found = letter(&tile).expect("the initial");
                let (inside, at) = (found.bounds, tile.bounds);
                let device = |v: f32| v * scale.factor();
                let whole = |v: f32| (device(v) - device(v).round()).abs() < 1e-3;

                assert!(
                    whole(inside.x - at.x) && whole(inside.y - at.y),
                    "{scale:?} {size:?} {name}: {inside:?}"
                );
                assert!(whole(inside.width), "{scale:?} {size:?} {name}: {inside:?}");
                assert!(
                    (inside.height - f32::from(size.initial().line())).abs() < 1e-3,
                    "{scale:?} {size:?} {name}: {inside:?}"
                );
                let left = device(inside.x - at.x);
                let right = device(at.x + at.width - inside.x - inside.width);
                let odd = left - right;
                assert!(
                    odd.abs() < 1e-3 || (odd - 1.0).abs() < 1e-3,
                    "{scale:?} {size:?} {name}: {left} {right}"
                );
            }
        }
    }
}

use std::borrow::Cow;

use iced::Element;
use iced::Pixels;
use iced::Renderer;
use iced::Size;
use iced::advanced::graphics::text::font_system;
use iced::advanced::renderer::Headless;
use iced_runtime::UserInterface;
use iced_runtime::user_interface;
use kubaos_geometry::Frame;
use kubaos_geometry::Node;
use kubaos_geometry::Role;
use kubaos_geometry::Rule;
use kubaos_geometry::Rules;

use super::Collect;
use crate::component::field::field;
use crate::layout::stack;
use crate::look::Look;
use crate::scale::Scale;
use crate::tokens;
use crate::tokens::Mode;
use crate::tokens::Space;

/// `view`'s geometry tree, laid out headless `width` wide.
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

fn fields(node: &mut Node) -> Vec<&mut Node> {
    let mut found = Vec::new();
    if matches!(&node.role, Role::Component { name, .. } if name == "Field") {
        found.push(node);
        return found;
    }
    for child in &mut node.children {
        found.extend(fields(child));
    }
    found
}

fn rules() -> Rules {
    Rules {
        grid: f32::from(tokens::GRID),
        spacing: Space::ALL.iter().map(|s| f32::from(s.px())).collect(),
    }
}

#[test]
fn a_field_without_an_icon_is_reported_as_a_field() {
    let look = Look::new(Mode::Dark, Scale::X1);

    let mut root = tree(
        stack(&look, Space::S8)
            .push(field::<()>("Kuba", "Your name").view(&look))
            .into(),
    );

    let found = fields(&mut root);
    assert_eq!(found.len(), 1, "{root:#?}");
    assert!((found[0].bounds.height - 40.0).abs() < 1e-4, "{:?}", found[0].bounds);
}

#[test]
fn a_plain_field_forty_one_tall_fails_the_geometry_check() {
    let look = Look::new(Mode::Dark, Scale::X1);
    let mut root = tree(
        stack(&look, Space::S8)
            .push(field::<()>("Kuba", "Your name").view(&look))
            .into(),
    );
    fields(&mut root)[0].bounds.height = 41.0;

    let frame = Frame {
        page: "field".to_owned(),
        mode: "dark".to_owned(),
        scale: 1.0,
        root,
    };
    let broken: Vec<Rule> = kubaos_geometry::check(&[frame], &rules())
        .into_iter()
        .map(|f| f.rule)
        .collect();

    assert!(broken.contains(&Rule::ScaleGrid), "{broken:?}");
}

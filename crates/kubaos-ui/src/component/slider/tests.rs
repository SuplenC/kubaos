use super::Direction;
use super::Value;
use super::stick;

fn stop(index: u16, count: u16) -> Value {
    Value::Stop { index, count }
}

#[test]
fn a_stepped_slider_settles_on_the_nearest_stop_when_released() {
    let power = stop(0, 5);

    assert_eq!(power.at(0.0), stop(0, 5));
    assert_eq!(power.at(0.12), stop(0, 5));
    assert_eq!(power.at(0.13), stop(1, 5));
    assert_eq!(power.at(0.6), stop(2, 5));
    assert_eq!(power.at(0.9), stop(4, 5));
}

#[test]
fn a_release_past_either_end_settles_on_that_end() {
    let power = stop(2, 5);

    assert_eq!(power.at(-0.4), stop(0, 5));
    assert_eq!(power.at(1.7), stop(4, 5));
}

#[test]
fn a_continuous_slider_takes_the_value_under_the_pointer_unsnapped() {
    assert_eq!(Value::Continuous(0.5).at(0.37), Value::Continuous(0.37));
    assert_eq!(Value::Continuous(0.5).at(1.2), Value::Continuous(1.0));
}

#[test]
fn the_icon_walks_the_stops_down_and_back_up_turning_at_each_end() {
    let mut at = (stop(2, 3), Direction::Down);
    let mut walked = Vec::new();

    for _ in 0..6 {
        at = at.0.turn(at.1);
        walked.push(at.0);
    }

    assert_eq!(
        walked,
        [stop(1, 3), stop(0, 3), stop(1, 3), stop(2, 3), stop(1, 3), stop(0, 3)]
    );
}

#[test]
fn the_icon_turns_around_at_the_top_whichever_way_it_was_going() {
    assert_eq!(stop(4, 5).turn(Direction::Up), (stop(3, 5), Direction::Down));
}

#[test]
fn the_icon_turns_around_at_the_bottom_whichever_way_it_was_going() {
    assert_eq!(stop(0, 5).turn(Direction::Down), (stop(1, 5), Direction::Up));
}

#[test]
fn the_icon_keeps_its_direction_between_the_ends() {
    assert_eq!(stop(2, 5).turn(Direction::Up), (stop(3, 5), Direction::Up));
    assert_eq!(stop(2, 5).turn(Direction::Down), (stop(1, 5), Direction::Down));
}

#[test]
fn the_icon_leaves_a_slider_without_stops_where_it_is() {
    assert_eq!(stop(0, 1).turn(Direction::Down), (stop(0, 1), Direction::Down));
    assert_eq!(
        Value::Continuous(0.4).turn(Direction::Up),
        (Value::Continuous(0.4), Direction::Up)
    );
}

#[test]
fn an_arrow_key_moves_a_stepped_slider_one_stop_and_stops_at_the_ends() {
    assert_eq!(stop(2, 5).step(true), stop(3, 5));
    assert_eq!(stop(2, 5).step(false), stop(1, 5));
    assert_eq!(stop(4, 5).step(true), stop(4, 5));
    assert_eq!(stop(0, 5).step(false), stop(0, 5));
}

#[test]
fn an_arrow_key_moves_a_continuous_slider_a_twentieth_within_its_range() {
    let Value::Continuous(up) = Value::Continuous(0.5).step(true) else {
        panic!("a continuous slider stays continuous");
    };
    assert!((up - 0.55).abs() < 1e-6);
    assert_eq!(Value::Continuous(0.98).step(true), Value::Continuous(1.0));
    assert_eq!(Value::Continuous(0.02).step(false), Value::Continuous(0.0));
}

#[test]
fn home_and_end_go_to_the_ends() {
    assert_eq!(stop(2, 5).end(false), stop(0, 5));
    assert_eq!(stop(2, 5).end(true), stop(4, 5));
    assert_eq!(Value::Continuous(0.3).end(true), Value::Continuous(1.0));
    assert_eq!(Value::Continuous(0.3).end(false), Value::Continuous(0.0));
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "`stick` gives back the point or the value itself, never a new float."
)]
fn a_drag_near_a_sticky_point_holds_on_it() {
    assert_eq!(stick(0.98, 1.0, 0.03), 1.0);
    assert_eq!(stick(1.025, 1.0, 0.03), 1.0);
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "`stick` gives back the point or the value itself, never a new float."
)]
fn a_drag_pulled_past_a_sticky_points_reach_leaves_it() {
    assert_eq!(stick(0.96, 1.0, 0.03), 0.96);
    assert_eq!(stick(1.04, 1.0, 0.03), 1.04);
}

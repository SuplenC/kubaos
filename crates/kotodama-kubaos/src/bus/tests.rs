use super::*;

#[test]
fn a_boolean_variant_reads_in_either_byte_order() {
    let little = u32::from_le_bytes([1, b'b', 0, 0]);
    let big = u32::from_be_bytes([1, b'b', 0, 0]);

    assert_eq!(bool_variant(little, 1), Some(true));
    assert_eq!(bool_variant(little, 0), Some(false));
    assert_eq!(bool_variant(big, 1), Some(true));
}

#[test]
fn a_variant_of_another_type_is_no_boolean() {
    let string = u32::from_le_bytes([1, b's', 0, 0]);
    let boolean = u32::from_le_bytes([1, b'b', 0, 0]);

    assert_eq!(bool_variant(string, 1), None);
    assert_eq!(bool_variant(boolean, 2), None);
}

#[test]
fn the_children_are_the_inner_nodes() {
    let xml = r#"<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
"http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node name="/org/bluez">
  <interface name="org.freedesktop.DBus.Introspectable">
    <method name="Introspect"><arg name="xml" type="s" direction="out"/></method>
  </interface>
  <node name="hci0"/>
  <node name="hci1"></node>
</node>"#;

    assert_eq!(child_nodes(xml), ["hci0", "hci1"]);
}

#[test]
fn an_object_without_children_has_none() {
    assert!(child_nodes("<node><interface name=\"a.b\"/></node>").is_empty());
}

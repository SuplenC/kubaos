use super::*;

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

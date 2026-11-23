import QtQuick
import QtTest
import KubaOS.Components as K
import "../../config/quickshell/modules/menu"

// The detail pane's mark, in its row's order: an Item's picture once it has loaded, else its
// Extension's symbol; a desktop app gets the lg tile with its initial instead, its row's tile at
// the pane's size, while its picture loads too.
TestCase {
    id: suite

    name: "MenuPane"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 400
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 600

    // A picture as a desktop entry names one, by its path; the stand-in `iconPath` finds no theme
    // icon, so a theme name is an app with no picture.
    readonly property string picture: Qt.resolvedUrl("../../branding/png/kubaos-icon-256.png").toString().replace("file://", "")

    PaneContent {
        id: pane

        width: suite.width
        height: suite.height
    }

    function entry(icon: string, extension: string, title: string): var {
        return {
            icon: icon,
            extension: extension,
            title: title,
            subtitle: "",
            unavailable: "",
            alias: "",
            actions: [],
            hints: ""
        };
    }

    function cleanup() {
        pane.entry = null;
    }

    function test_a_desktop_app_with_no_picture_shows_its_initial_on_the_lg_tile() {
        pane.entry = suite.entry("krita", "apps", "Krita");
        const tile = findChild(pane, "initialTile");

        verify(tile !== null);
        compare(tile.size, "Lg");
        compare(tile.letter, "K");
        compare(tile.width, K.Tokens.size.control_lg);
        verify(!findChild(pane, "symbol").visible, "no bolt, no desktop symbol");
    }

    function test_an_item_with_no_icon_shows_its_extension_s_symbol() {
        pane.entry = suite.entry("", "calculator", "2 + 2");

        compare(findChild(pane, "symbol").name, "calculate");
        compare(findChild(pane, "initialTile"), null);
    }

    function test_the_initial_holds_the_place_until_the_app_s_picture_has_loaded() {
        pane.entry = suite.entry(suite.picture, "apps", "KubaOS");

        verify(findChild(pane, "initialTile") !== null, "the initial while the picture loads");
        tryVerify(() => findChild(pane, "initialTile") === null, 5000, "the picture takes its place once it has loaded");
        verify(!findChild(pane, "symbol").visible);
    }
}

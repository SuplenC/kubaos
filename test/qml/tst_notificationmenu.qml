import QtQuick
import QtTest
import qs.services
import "../../config/quickshell/modules/notifications"

// A notification's menu in the notifications panel goes with the panel, however the panel closes.
TestCase {
    id: suite

    name: "NotificationMenu"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 600
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 600

    property bool panelShown: true

    Item {
        id: panel

        anchors.fill: parent

        NotificationMenu {
            id: menu

            held: suite.panelShown
        }
    }

    function init() {
        suite.panelShown = true;
        menu.close();
    }

    function test_a_menu_opens_over_its_panel() {
        menu.open(Notifs.history[0], Qt.point(0, 0));

        verify(menu.visible);
    }

    function test_the_panel_closing_takes_its_menu_with_it() {
        menu.open(Notifs.history[0], Qt.point(0, 0));

        suite.panelShown = false;

        compare(menu.notification, null);
        verify(!menu.visible);
    }
}

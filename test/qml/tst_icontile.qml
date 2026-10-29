import QtQuick
import QtTest
import KubaOS.Components as K

// An icon tile with a photo: the symbol shows until the photo has loaded, and the photo's
// rounded mask is there from the start, so the photo draws in the same frame it loads.
TestCase {
    id: suite

    name: "IconTile"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 200
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 200

    // A picture for each test: Qt keeps one loaded, and a second load of it is ready at once.
    readonly property url photo: Qt.resolvedUrl("../../branding/png/kubaos-icon-1024.png")
    readonly property url other: Qt.resolvedUrl("../../branding/png/kubaos-horizontal.png")

    K.IconTile {
        id: tile

        size: "Sm"
    }

    function cleanup() {
        tile.photo = "";
    }

    // Regression: #252.
    function test_a_photo_s_mask_is_there_before_the_photo_has_loaded() {
        tile.photo = suite.photo;
        const cover = findChild(tile, "cover");

        verify(cover.status !== Image.Ready, "still loading, as it loads asynchronously");
        verify(findChild(tile, "corners").layer.enabled);
        tryCompare(cover, "status", Image.Ready);
    }

    function test_the_symbol_shows_until_the_photo_has_loaded() {
        tile.photo = suite.other;

        verify(!tile.pictured);
        tryVerify(() => tile.pictured, 5000, "the photo takes the tile once it has loaded");
    }
}

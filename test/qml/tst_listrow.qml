import QtQuick
import QtTest
import KubaOS.Components as K

// A row still being found keeps the place of the tiled two-line row that will take it, and takes
// no press while it waits. The Command menu's options lay a Result out as its rows always have, and
// a row stays cheap enough for the menu to make fourteen on a keystroke: no layout primitives, and
// each text measured once as it is made.
TestCase {
    id: suite

    name: "ListRow"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 300
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 200

    property int presses: 0
    property var points: []

    K.ListRow {
        id: row

        width: suite.width
        title: "Home-5G"
        onActivated: suite.presses++
    }

    K.ListRow {
        id: found

        y: K.Tokens.size.row_two_line
        width: suite.width
        title: "Home-5G"
        subtitle: "Secured"
        icon: "wifi"
        tile: true
    }

    Component {
        id: result

        K.ListRow {
            width: suite.width
            title: "Firefox Web Browser"
            subtitle: "Browse the web"
            icon: "desktop_windows"
            tile: true
            tall: true
            clear: true
            tight: true
            fills: false
            hoverLayer: false
        }
    }

    // The first item under `item`, itself included, that `test` holds for.
    function find(item, test) {
        if (test(item))
            return item;
        for (const child of item.children) {
            const hit = suite.find(child, test);
            if (hit)
                return hit;
        }
        return null;
    }

    // Every item under `item`, itself included, that `test` holds for.
    function findAll(item, test) {
        const found = test(item) ? [item] : [];
        for (const child of item.children)
            found.push(...suite.findAll(child, test));
        return found;
    }

    function textOf(item, words) {
        return suite.find(item, child => child.probe?.role === "text" && child.text === words);
    }

    function init() {
        row.loading = false;
        suite.presses = 0;
        suite.points = [];
    }

    function test_a_loading_row_is_as_tall_as_a_two_line_row() {
        compare(row.implicitHeight, K.Tokens.size.row);

        row.loading = true;

        compare(row.implicitHeight, K.Tokens.size.row_two_line);
    }

    function test_a_loading_row_takes_no_press() {
        row.loading = true;

        mouseClick(row);

        compare(suite.presses, 0);
    }

    function test_a_loading_rows_skeletons_sit_where_the_found_rows_tile_and_title_do() {
        row.loading = true;
        tryVerify(() => suite.find(row, item => item.bone === "block") !== null);

        const bone = suite.find(row, item => item.bone === "block");
        const line = suite.find(row, item => item.bone === "line");
        const tile = suite.find(found, item => item.probe?.name === "IconTile");
        const title = suite.find(found, item => item.text === "Home-5G");
        const boneAt = bone.mapToItem(row, 0, 0);
        const tileAt = tile.mapToItem(found, 0, 0);
        compare(boneAt.x, tileAt.x);
        compare(boneAt.y, tileAt.y);
        compare(bone.width, tile.width);
        compare(line.mapToItem(row, 0, 0).x, title.mapToItem(found, 0, 0).x);
    }

    function test_a_loaded_row_takes_a_press() {
        mouseClick(row);

        compare(suite.presses, 1);
    }

    function test_a_tall_row_is_two_lines_tall_on_a_regular_tile_with_its_title_alone() {
        const made = createTemporaryObject(result, suite, {
            subtitle: ""
        });

        compare(made.implicitHeight, K.Tokens.size.row_two_line);
        const tile = suite.find(made, item => item.probe?.name === "IconTile");
        compare(tile.probe.variant, "Md");
        const title = suite.textOf(made, "Firefox Web Browser");
        compare(title.mapToItem(made, 0, 0).y, (K.Tokens.size.row_two_line - K.Tokens.type.body_strong.line) / 2);
    }

    function test_clear_words_start_16_after_the_tile() {
        const made = createTemporaryObject(result, suite);

        const title = suite.textOf(made, "Firefox Web Browser");
        compare(title.mapToItem(made, 0, 0).x, K.Tokens.space.s12 + K.Tokens.size.control_md + K.Tokens.space.s16);
    }

    function test_a_tight_subtitle_sits_right_under_the_title() {
        const made = createTemporaryObject(result, suite);

        const title = suite.textOf(made, "Firefox Web Browser");
        const subtitle = suite.textOf(made, "Browse the web");
        compare(subtitle.mapToItem(made, 0, 0).y, title.mapToItem(made, 0, 0).y + K.Tokens.type.body_strong.line);
        compare(title.mapToItem(made, 0, 0).y, (K.Tokens.size.row_two_line - K.Tokens.type.body_strong.line - K.Tokens.type.small.line) / 2);
    }

    function test_a_row_the_list_selects_draws_no_fill_and_takes_the_selections_colours() {
        const made = createTemporaryObject(result, suite, {
            selected: true
        });

        const fills = suite.findAll(made, item => item.visible && item.color !== undefined && Qt.colorEqual(item.color, K.Tokens.color.secondary_container));
        compare(fills.length, 0);
        verify(Qt.colorEqual(suite.textOf(made, "Firefox Web Browser").color, K.Tokens.color.on_secondary_container));
    }

    function test_a_row_with_no_hover_layer_makes_none() {
        const made = createTemporaryObject(result, suite);

        compare(suite.find(made, item => item.content !== undefined && item.pressed !== undefined), null);
        verify(suite.find(row, item => item.content !== undefined && item.pressed !== undefined) !== null);
    }

    function test_an_unavailable_row_dims_its_tile_and_title_but_not_its_subtitle() {
        const made = createTemporaryObject(result, suite, {
            available: false
        });

        const tile = suite.find(made, item => item.probe?.name === "IconTile");
        compare(tile.opacity, K.Tokens.opacity.disabled_content);
        compare(suite.textOf(made, "Firefox Web Browser").color.a, K.Tokens.opacity.disabled_content);
        compare(suite.textOf(made, "Browse the web").color.a, 1);
    }

    function test_an_unavailable_row_still_takes_a_press() {
        const made = createTemporaryObject(result, suite, {
            available: false
        });
        made.activated.connect(() => suite.presses++);

        mouseClick(made);

        compare(suite.presses, 1);
    }

    function test_the_titles_lit_matches_are_drawn() {
        const made = createTemporaryObject(result, suite, {
            markup: "<b>Fi</b>refox Web Browser"
        });

        verify(suite.textOf(made, "Firefox Web Browser").lit);
    }

    function test_the_pointer_moving_over_a_row_says_where() {
        const made = createTemporaryObject(result, suite);
        made.pointed.connect((x, y) => suite.points.push([x, y]));

        mouseMove(made, 30, 20);

        verify(suite.points.length > 0);
        compare(suite.points[suite.points.length - 1], [30, 20]);
    }

    function test_a_row_lays_out_with_no_layout_primitive() {
        const passes = K.LayoutPasses.count;

        const made = createTemporaryObject(result, suite, {
            detail: "ff"
        });

        compare(suite.find(made, item => typeof item.arrange === "function"), null);
        compare(K.LayoutPasses.count, passes);
    }

    function test_a_rows_texts_are_each_measured_once_as_it_is_made() {
        const title = `Firefox Web Browser ${Date.now()}`;
        const subtitle = `Browse the web ${Date.now()}`;
        const before = K.Advances.widths.size;

        createTemporaryObject(result, suite, {
            title,
            subtitle
        });

        compare(K.Advances.widths.size, before + 2);
    }
}

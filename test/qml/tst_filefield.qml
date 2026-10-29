import QtQuick
import QtTest
import Kotodama
import "../../config/quickshell/modules/menu"

// A Command menu form's file and folder field: what it asks the Daemon as it is typed into, and
// the keys of the list of paths under it.
TestCase {
    id: suite

    name: "FileField"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 600
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 600

    readonly property var downloads: [
        {
            path: "/home/kuba/Downloads/work-kuba.conf",
            title: "work-kuba.conf",
            folder: false,
            modified: 1793280000,
            highlights: []
        },
        {
            path: "/home/kuba/Downloads/wireguard",
            title: "wireguard",
            folder: true,
            modified: -1,
            highlights: []
        },
        {
            path: "/home/kuba/Downloads/work-old.conf",
            title: "work-old.conf",
            folder: false,
            modified: 1790000000,
            highlights: []
        }
    ]

    KotodamaSearch {
        id: engine
    }

    // Each key the field passed on that the form and the menu take: they move between fields,
    // submit and go back.
    property var passed: []
    property int browsed: 0
    readonly property var formKeys: [Qt.Key_Tab, Qt.Key_Backtab, Qt.Key_Return, Qt.Key_Enter, Qt.Key_Escape, Qt.Key_Up, Qt.Key_Down]

    Component {
        id: fileField

        FileField {
            // design-allow(size): a test's fixture, not a design value.
            width: 480
            search: engine
            place: 0
            keys: event => {
                if (!suite.formKeys.includes(event.key))
                    return false;
                suite.passed.push(event.key);
                return true;
            }
            onBrowse: suite.browsed += 1
        }
    }

    Component {
        id: formView

        FormView {
            // design-allow(size): a test's fixture, not a design value.
            width: 480
            // design-allow(size): a test's fixture, not a design value.
            height: 400
            search: engine
            fields: [
                {
                    label: "Name",
                    kind: "text",
                    placeholder: "",
                    value: ""
                },
                {
                    label: "Config",
                    kind: "file",
                    accept: ["conf", "ovpn"],
                    placeholder: "~/Downloads/work.conf",
                    value: ""
                },
                {
                    label: "Into",
                    kind: "folder",
                    accept: [],
                    placeholder: "",
                    value: ""
                }
            ]
            submitLabel: "Import"
        }
    }

    SignalSpy {
        id: browseSpy

        signalName: "browse"
    }

    // The items under `item` for which `test` holds, depth first.
    function all(item, test) {
        const found = test(item) ? [item] : [];
        for (const kid of item.children)
            found.push(...all(kid, test));
        return found;
    }

    function init() {
        engine.completions = [];
        engine.completingField = -1;
        engine.lastCompleted = "";
        engine.lastText = "";
        suite.passed = [];
        suite.browsed = 0;
    }

    // A field with the keys, the Daemon's answer `paths` in.
    function focused(paths, folder) {
        const field = createTemporaryObject(fileField, suite, {
            folder: folder ?? false
        });
        field.focusField();
        engine.completions = paths ?? suite.downloads;
        waitForRendering(field);
        return field;
    }

    function test_with_the_keys_and_nothing_typed_it_asks_for_the_newest_files() {
        const field = focused();

        compare(engine.lastCompleted, "0:");
        verify(field.listOpen, "the newest files show under the field");
        compare(field.count, 3);
    }

    function test_typing_asks_the_daemon_for_what_is_typed() {
        focused();

        keyClick("~");
        keyClick("/");
        keyClick("w");

        compare(engine.lastCompleted, "0:~/w");
        compare(engine.lastText, "0:~/w");
    }

    function test_down_and_up_pick_a_path_and_up_from_the_first_picks_none() {
        const field = focused();

        keyClick(Qt.Key_Down);
        keyClick(Qt.Key_Down);
        compare(field.picked, 1);
        keyClick(Qt.Key_Up);
        keyClick(Qt.Key_Up);
        compare(field.picked, -1);
        compare(suite.passed, [], "the list took every arrow");
    }

    function test_down_stops_at_the_last_path() {
        const field = focused();

        for (let n = 0; n < 5; n++)
            keyClick(Qt.Key_Down);

        compare(field.picked, 2);
    }

    function test_ctrl_n_and_ctrl_p_move_as_the_arrows_do() {
        const field = focused();

        keyClick(Qt.Key_N, Qt.ControlModifier);
        keyClick(Qt.Key_N, Qt.ControlModifier);
        keyClick(Qt.Key_P, Qt.ControlModifier);

        compare(field.picked, 0);
    }

    function test_enter_takes_the_picked_file_home_relative_and_closes_the_list() {
        const field = focused();

        keyClick(Qt.Key_Down);
        keyClick(Qt.Key_Return);

        compare(field.text, "~/Downloads/work-kuba.conf");
        verify(!field.listOpen, "the list closes once a path is taken");
        compare(suite.passed, [], "Enter took the path; it didn't submit");
    }

    function test_enter_with_nothing_picked_submits() {
        focused();

        keyClick(Qt.Key_Return);

        compare(suite.passed, [Qt.Key_Return]);
    }

    function test_a_path_typed_in_part_keeps_its_folder_as_typed() {
        const field = focused();
        field.text = "~/Downloads/wo";

        keyClick(Qt.Key_Down);
        keyClick(Qt.Key_Return);

        compare(field.text, "~/Downloads/work-kuba.conf");
    }

    function test_tab_on_a_folder_goes_into_it_and_goes_on_completing() {
        const field = focused();
        field.text = "~/Downloads/w";

        keyClick(Qt.Key_Down);
        keyClick(Qt.Key_Down);
        keyClick(Qt.Key_Tab);

        compare(field.text, "~/Downloads/wireguard/");
        compare(engine.lastCompleted, "0:~/Downloads/wireguard/");
        verify(field.listOpen, "the folder's paths show next");
    }

    function test_tab_with_nothing_picked_completes_the_best_path() {
        const field = focused();
        field.text = "~/Downloads/w";

        keyClick(Qt.Key_Tab);

        compare(field.text, "~/Downloads/work-kuba.conf");
    }

    function test_tab_with_nothing_typed_or_picked_moves_on() {
        focused();

        keyClick(Qt.Key_Tab);

        compare(suite.passed, [Qt.Key_Tab]);
    }

    function test_enter_on_a_folder_in_a_file_field_goes_into_it() {
        const field = focused();

        keyClick(Qt.Key_Down);
        keyClick(Qt.Key_Down);
        keyClick(Qt.Key_Return);

        compare(field.text, "~/Downloads/wireguard/");
        verify(field.listOpen, "a file field wants a file, so the folder opens");
    }

    function test_enter_on_a_folder_in_a_folder_field_takes_it() {
        const field = focused([suite.downloads[1]], true);

        keyClick(Qt.Key_Down);
        keyClick(Qt.Key_Return);

        compare(field.text, "~/Downloads/wireguard");
        verify(!field.listOpen);
    }

    function test_esc_closes_the_list_and_nothing_else() {
        const field = focused();

        keyClick(Qt.Key_Escape);

        verify(!field.listOpen);
        compare(suite.passed, [], "Esc closed the list before it went back");
        keyClick(Qt.Key_Escape);
        compare(suite.passed, [Qt.Key_Escape], "with the list closed, Esc goes back");
    }

    function test_typing_after_esc_opens_the_list_again() {
        const field = focused();
        keyClick(Qt.Key_Escape);

        keyClick("w");

        verify(field.listOpen);
    }

    function test_arrows_with_the_list_closed_move_between_fields() {
        focused([]);

        keyClick(Qt.Key_Down);

        compare(suite.passed, [Qt.Key_Down]);
    }

    function test_ctrl_o_opens_the_picker() {
        focused();

        keyClick(Qt.Key_O, Qt.ControlModifier);

        compare(suite.browsed, 1);
        compare(suite.passed, []);
    }

    function test_the_list_shows_five_paths_at_most() {
        const many = Array.from({
                length: 8
            }, (_, n) => Object.assign({}, suite.downloads[0], {
                title: `work-${n}.conf`,
                path: `/home/kuba/Downloads/work-${n}.conf`
            }));
        const field = focused(many);

        for (let n = 0; n < 8; n++)
            keyClick(Qt.Key_Down);

        compare(field.count, 5);
        compare(field.picked, 4);
    }

    function test_answers_for_another_field_show_nothing() {
        const field = focused();

        engine.completingField = 1;

        verify(!field.listOpen);
    }

    function test_each_path_shows_its_folder_home_relative() {
        const field = focused();

        compare(field.folderOf(suite.downloads[0].path), "~/Downloads");
        compare(field.folderOf("/etc/hosts"), "/etc");
        compare(field.folderOf("/hosts"), "/");
    }

    function test_browse_in_a_form_asks_for_a_picker_of_the_fields_kinds() {
        const form = createTemporaryObject(formView, suite);
        browseSpy.target = form;
        waitForRendering(form);
        const buttons = all(form, item => item.text === "Browse…" && typeof item.activated === "function" && item.visible);

        compare(buttons.length, 2, "a Browse… beside each file and folder field, none beside text");
        mouseClick(buttons[0]);
        mouseClick(buttons[1]);

        compare(browseSpy.count, 2);
        compare(browseSpy.signalArguments[0][0], 1);
        compare(browseSpy.signalArguments[0][1], false);
        compare(browseSpy.signalArguments[0][2], ["conf", "ovpn"]);
        compare(browseSpy.signalArguments[1][0], 2);
        compare(browseSpy.signalArguments[1][1], true);
    }

    function test_the_path_the_picker_chose_fills_its_field_and_takes_the_keys() {
        const form = createTemporaryObject(formView, suite);
        waitForRendering(form);

        form.fill(1, "/home/kuba/Downloads/work.conf");

        compare(engine.lastText, "1:~/Downloads/work.conf");
        compare(form.current, 1);
        verify(form.pathCurrent);
        verify(!form.pathsOpen, "the path is taken: no list over it");
    }

    function test_a_path_is_never_put_in_a_text_field() {
        const form = createTemporaryObject(formView, suite);
        waitForRendering(form);

        form.fill(0, "/home/kuba/Downloads/work.conf");

        compare(engine.lastText, "");
    }
}

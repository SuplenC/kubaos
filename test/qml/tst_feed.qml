import QtQuick
import QtTest
import "../../config/quickshell/modules/notifications/feed.js" as Feed

// What the notifications panel, the bell and its peek show of the history: stacks by app, New and
// Earlier, the three most recent unread for the peek, and whether the bell shows at all.
TestCase {
    id: suite

    name: "Feed"

    // Newest first, as `Notifs.history` is.
    function note(id, app) {
        return {
            id: id,
            appName: app
        };
    }

    function unreadOf(ids) {
        return n => ids.includes(n.id);
    }

    function idsOf(stack) {
        return stack.items.map(n => n.id);
    }

    function test_several_from_one_app_stack_into_one_with_the_newest_first() {
        const history = [note(5, "Slack"), note(4, "Mail"), note(3, "Slack"), note(2, "Slack")];

        const stacks = Feed.stacks(history);

        compare(stacks.length, 2);
        compare(stacks[0].app, "Slack");
        compare(idsOf(stacks[0]), [5, 3, 2]);
        compare(idsOf(stacks[1]), [4]);
    }

    function test_a_stack_takes_the_place_of_its_newest() {
        const history = [note(3, "Mail"), note(2, "Slack"), note(1, "Mail")];

        compare(Feed.stacks(history).map(s => s.app), ["Mail", "Slack"]);
    }

    function test_an_app_without_a_name_stacks_by_its_desktop_entry() {
        const history = [{
                id: 2,
                appName: "",
                desktopEntry: "org.gnome.Nautilus"
            }, {
                id: 1,
                appName: "",
                desktopEntry: "org.gnome.Nautilus"
            }];

        compare(Feed.stacks(history).length, 1);
    }

    function test_unread_ones_are_new_and_the_rest_earlier() {
        const history = [note(4, "Slack"), note(3, "Mail"), note(2, "Slack"), note(1, "Mail")];

        const sections = Feed.sections(history, unreadOf([4, 2]));

        compare(sections.fresh.map(idsOf), [[4, 2]]);
        compare(sections.earlier.map(idsOf), [[3, 1]]);
    }

    function test_the_peek_shows_the_three_most_recent_unread() {
        const history = [note(6, "A"), note(5, "B"), note(4, "C"), note(3, "D"), note(2, "E"), note(1, "F")];

        const peek = Feed.peek(history, unreadOf([6, 5, 4, 3, 1]));

        compare(peek.stacks.map(idsOf), [[6], [5], [4]]);
        compare(peek.more, 2);
    }

    function test_the_peek_leaves_out_what_is_read() {
        const history = [note(3, "A"), note(2, "B"), note(1, "C")];

        const peek = Feed.peek(history, unreadOf([3, 1]));

        compare(peek.stacks.map(idsOf), [[3], [1]]);
        compare(peek.more, 0);
    }

    function test_the_peek_stacks_by_app_so_a_burst_is_one_card() {
        const history = [note(5, "Slack"), note(4, "Slack"), note(3, "Mail"), note(2, "Slack"), note(1, "Slack")];

        const peek = Feed.peek(history, unreadOf([5, 4, 3, 2, 1]));

        compare(peek.stacks.map(idsOf), [[5, 4, 2, 1], [3]]);
        compare(peek.more, 0);
    }

    function test_more_counts_the_unread_past_the_three_cards() {
        const history = [note(6, "A"), note(5, "A"), note(4, "B"), note(3, "C"), note(2, "D"), note(1, "D")];

        const peek = Feed.peek(history, unreadOf([6, 5, 4, 3, 2, 1]));

        compare(peek.stacks.length, 3);
        compare(peek.more, 2, "D's two are past the three cards");
    }

    function test_with_nothing_unread_there_is_no_peek() {
        const peek = Feed.peek([note(1, "A")], unreadOf([]));

        compare(peek.stacks.length, 0);
        compare(peek.more, 0);
    }

    function test_the_bell_shows_only_while_there_are_notifications() {
        verify(!Feed.bellShown(0, false));
        verify(Feed.bellShown(1, false));
    }

    function test_with_always_show_the_bell_it_shows_with_none() {
        verify(Feed.bellShown(0, true));
        verify(Feed.bellShown(3, true));
    }

    function test_rows_list_the_sections_with_a_label_each() {
        const sections = Feed.sections([note(2, "A"), note(1, "B")], unreadOf([2]));

        const rows = Feed.rows(sections, {});

        compare(rows.map(r => r.kind), ["label", "card", "label", "card"]);
        compare(rows[0].text, "New");
        compare(rows[2].text, "Earlier");
    }

    function test_rows_leave_out_an_empty_section() {
        const rows = Feed.rows(Feed.sections([note(1, "A")], unreadOf([])), {});

        compare(rows.map(r => r.kind), ["label", "card"]);
        compare(rows[0].text, "Earlier");
    }

    function test_a_collapsed_stack_is_one_row_and_an_expanded_one_a_row_each() {
        const sections = Feed.sections([note(3, "A"), note(2, "A"), note(1, "B")], unreadOf([3, 2, 1]));

        const collapsed = Feed.rows(sections, {});
        compare(collapsed.length, 3);
        compare(collapsed[1].items.length, 2);
        verify(collapsed[1].stacked);

        const expanded = Feed.rows(sections, {
            [collapsed[1].key]: true
        });
        compare(expanded.length, 4);
        compare(expanded.slice(1, 3).map(r => r.items[0].id), [3, 2]);
        verify(!expanded[1].stacked);
        compare(expanded[1].key, collapsed[1].key, "an expanded card still names its stack, so ← folds it");
    }

    function test_the_status_line_says_how_many_are_new() {
        compare(Feed.status(2, 5, ""), "2 new");
        compare(Feed.status(0, 5, ""), "All read");
        compare(Feed.status(0, 0, ""), "Inbox zero");
    }

    function test_the_status_line_says_why_it_is_silenced() {
        compare(Feed.status(2, 5, "sharing"), "Silenced while you share your screen");
        compare(Feed.status(2, 5, "fullscreen"), "Silenced while a window is fullscreen");
        compare(Feed.status(2, 5, "manual"), "Silenced until you turn it back on");
        compare(Feed.status(2, 5, "overridden"), "2 new", "turned off during a share, it isn't silenced");
    }

    function test_the_do_not_disturb_button_says_why_it_is_on() {
        verify(Feed.dndLabel("sharing").includes("while you share your screen"));
        verify(Feed.dndLabel("fullscreen").includes("while a window is fullscreen"));
        verify(Feed.dndLabel("manual").includes("until you turn it off"));
        verify(Feed.dndLabel("overridden").includes("off until this ends"));
        verify(Feed.dndLabel("") !== Feed.dndLabel("manual"));
    }

    function test_the_same_app_stacks_apart_in_new_and_earlier() {
        const rows = Feed.rows(Feed.sections([note(2, "A"), note(1, "A")], unreadOf([2])), {});

        compare(rows.filter(r => r.kind === "card").length, 2);
        verify(rows[1].key !== rows[3].key);
    }
}

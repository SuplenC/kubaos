import QtQuick
import QtTest
import "../../config/quickshell/modules/counters/counters.js" as Logic

// What the counters' sheet says and checks: the value box's width, the name a new counter is
// offered, why a name, count, stop or rule can't be taken, what an Auto reset box reads, a
// counter's menu and Auto reset's choices, and how its history's rows read.
TestCase {
    id: suite

    name: "Counters"

    function counter(name, value, extra) {
        return Object.assign({
            id: 1,
            name: name,
            value: value,
            stop: 0,
            rule: "",
            next: 0,
            detail: ""
        }, extra ?? {});
    }

    readonly property var formats: ({
            date: "ddd d MMM",
            time: "HH:mm",
            week: "d MMM",
            month: "MMMM yyyy"
        })
    // Tuesday 27 October 2026, 21:47.
    readonly property date now: new Date(2026, 9, 27, 21, 47)

    function seconds(date) {
        return Math.floor(date.getTime() / 1000);
    }

    function test_the_value_box_is_as_wide_as_the_widest_value_or_stop() {
        compare(Logic.widest([]), "0");
        compare(Logic.widest([counter("a", 3), counter("b", 120)]), "000");
        compare(Logic.widest([counter("a", 7, {
                stop: 1000
            })]), "0000");
    }

    function test_a_new_counter_is_offered_the_first_free_name() {
        compare(Logic.nextName([]), "Counter");
        compare(Logic.nextName([counter("counter", 0)]), "Counter 2");
        compare(Logic.nextName([counter("Counter", 0), counter("Counter 2", 0)]), "Counter 3");
    }

    function test_a_name_can_t_be_empty_or_another_counter_s() {
        const counters = [counter("Coffee", 3), counter("Push-ups", 120)];

        compare(Logic.checkName("  ", counters, null), "A name can't be empty.");
        compare(Logic.checkName("push-ups", counters, null), "There's a counter named push-ups already.");
        compare(Logic.checkName("Tea", counters, null), "");
    }

    function test_a_counter_keeps_its_own_name_in_any_case() {
        const counters = [counter("Coffee", 3)];

        compare(Logic.checkName("Coffee", counters, "Coffee"), "");
        compare(Logic.checkName("COFFEE", counters, "Coffee"), "");
    }

    function test_a_value_is_a_whole_number_and_a_stop_one_or_more() {
        compare(Logic.checkCount("12"), "");
        compare(Logic.checkCount(" 0 "), "");
        compare(Logic.checkCount("3.5"), "A whole number, 0 or more.");
        compare(Logic.checkCount("-1"), "A whole number, 0 or more.");
        compare(Logic.checkStop("8"), "");
        compare(Logic.checkStop("0"), "A whole number, 1 or more.");
        compare(Logic.checkStop(""), "A whole number, 1 or more.");
    }

    function test_a_rule_read_says_how_it_resets_and_when_next() {
        const reading = {
            typed: "every day at 9",
            ok: true,
            words: "Every day at 9:00",
            when: "Next tomorrow at 9:00"
        };

        compare(Logic.ruleNote("every day at 9", reading), "Resets every day at 9:00. Next tomorrow at 9:00.");
        compare(Logic.checkRule("every day at 9", reading), "");
    }

    function test_a_rule_not_read_offers_the_phrases_and_is_turned_down() {
        const reading = {
            typed: "soon",
            ok: false,
            examples: ["every day at 9", "mondays 8am"]
        };

        compare(Logic.ruleNote("soon", reading), "Try “every day at 9”, “mondays 8am”");
        compare(Logic.checkRule("soon", reading), "Can't read that. Try “every day at 9”, “mondays 8am”");
        compare(Logic.checkRule("", reading), "Try “every day at 9”, “mondays 8am”");
    }

    function test_a_reading_of_other_words_is_never_taken_for_what_is_typed() {
        const reading = {
            typed: "every day at 9",
            ok: true,
            words: "Every day at 9:00",
            when: "Next tomorrow at 9:00",
            examples: ["every day at 9"]
        };

        verify(Logic.checkRule("every day at 9 and", reading) !== "");
    }

    function test_reset_now_is_offered_only_above_zero() {
        compare(Logic.menuEntries(counter("Counter", 0))[1].enabled, false);
        compare(Logic.menuEntries(counter("Coffee", 3))[1].enabled, true);
    }

    function test_the_menu_ends_in_a_held_remove_after_a_separator() {
        const entries = Logic.menuEntries(counter("Coffee", 3));

        compare(entries.length, Logic.MENU.length);
        verify(entries[4].separator);
        verify(entries[5].held);
        compare(Logic.MENU[5], "remove");
    }

    function test_auto_reset_ticks_never_or_its_stop() {
        const never = Logic.autoEntries(counter("Counter", 0));
        const stopped = Logic.autoEntries(counter("Water", 7, {
            stop: 8
        }));
        const ruled = Logic.autoEntries(counter("Coffee", 3, {
            rule: "every day at 9:00"
        }));

        verify(never[0].checked);
        compare(never[1].text, "When it reaches");
        compare(never[1].value, "10");
        verify(!stopped[0].checked);
        verify(stopped[1].checked);
        compare(stopped[1].value, "8");
        verify(!ruled[0].checked && !ruled[1].checked);
    }

    function test_a_change_reads_its_day_time_kind_and_value() {
        const today = Logic.historyLine({
            at: seconds(new Date(2026, 9, 27, 14, 22)),
            label: "+1",
            value: 3,
            added: 1,
            changes: 1
        }, "change", now, formats);
        const earlier = Logic.historyLine({
            at: seconds(new Date(2026, 9, 26, 9, 5)),
            label: "Set",
            value: 12,
            added: 0,
            changes: 1
        }, "change", now, formats);

        compare(today.title, "Today · 14:22");
        compare(today.what, "+1");
        compare(today.value, 3);
        compare(today.sign, "");
        compare(earlier.title, `${Qt.formatDate(new Date(2026, 9, 26), "ddd d MMM")} · 09:05`);
    }

    function test_a_group_reads_its_span_its_changes_and_what_they_added() {
        const day = Logic.historyLine({
            at: seconds(new Date(2026, 9, 27)),
            value: 3,
            added: 4,
            changes: 5
        }, "day", now, formats);
        const week = Logic.historyLine({
            at: seconds(new Date(2026, 9, 26)),
            value: 3,
            added: -2,
            changes: 1
        }, "week", now, formats);
        const month = Logic.historyLine({
            at: seconds(new Date(2026, 9, 1)),
            value: 3,
            added: 39,
            changes: 55
        }, "month", now, formats);

        compare(day.title, "Today");
        compare(day.what, "5 changes");
        compare(day.sign + day.value, "+4");
        compare(week.title, `Week of ${Qt.formatDate(new Date(2026, 9, 26), "d MMM")}`);
        compare(week.what, "1 change");
        compare(week.sign + week.value, "−2");
        compare(month.title, Qt.formatDate(new Date(2026, 9, 1), "MMMM yyyy"));
    }

    function test_most_added_is_the_grouped_sort_s_name() {
        compare(Logic.sortLabels("change")[2], "Highest");
        compare(Logic.sortLabels("week")[2], "Most added");
    }

    function test_the_history_s_value_box_fits_its_widest_row() {
        compare(Logic.historyWidest([]), "0");
        compare(Logic.historyWidest([{
                value: 7
            }, {
                value: 120
            }]), "000");
    }
}

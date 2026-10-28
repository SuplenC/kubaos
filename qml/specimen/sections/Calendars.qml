import QtQuick
import KubaOS.Components as K

// The components page's "Calendars" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/calendars.rs` draws with `kubaos-ui`, box for box.
K.Section {
    id: root

    // The specimen's October 2026, weeks from Monday, as the shots' clock has it: Tuesday the 27th
    // is today, and All Saints' Day falls in the next month's first week, as the iced page has it.
    readonly property var october: {
        const pad = n => String(n).padStart(2, "0");
        const weeks = [];
        for (let week = 0; week < 6; week++) {
            const days = [];
            for (let weekday = 0; weekday < 7; weekday++) {
                const index = week * 7 + weekday;
                const [month, day] = index <= 2 ? [9, 28 + index] : index <= 33 ? [10, index - 2] : [11, index - 33];
                days.push({
                    date: `2026-${pad(month)}-${pad(day)}`,
                    day: day,
                    inMonth: month === 10,
                    today: month === 10 && day === 27,
                    weekend: weekday >= 5,
                    holiday: month === 11 && day === 1 ? "All Saints' Day" : ""
                });
            }
            weeks.push({
                number: 40 + week,
                days: days
            });
        }
        return {
            year: 2026,
            month: 10,
            title: "October 2026",
            weekdays: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
            weeks: weeks
        };
    }

    title: "Calendars"
    description: "A month as six weeks led by their week numbers, today filled and the picked day ringed; the title opens the year."

    K.Row {
        gap: K.Tokens.space.s48

        K.Calendar {
            month: root.october
            selected: "2026-10-30"
        }

        K.Calendar {
            month: root.october
            yearView: true
        }
    }
}

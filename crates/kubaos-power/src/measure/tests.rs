use std::cell::Cell;

use super::*;
use crate::fake::FakeRoot;

const INTERRUPTS_BEFORE: &str = "\
           CPU0       CPU1
  0:         10          0  IR-IO-APIC    2-edge      timer
 24:        100        200  IR-PCI-MSIX-0000:c4:00.0    0-edge      amdgpu
LOC:       5000       6000   Local timer interrupts
ERR:          0
";

const INTERRUPTS_AFTER: &str = "\
           CPU0       CPU1
  0:         10          0  IR-IO-APIC    2-edge      timer
 24:        400        500  IR-PCI-MSIX-0000:c4:00.0    0-edge      amdgpu
LOC:       9000      12000   Local timer interrupts
ERR:          0
";

#[test]
fn interrupt_lines_are_summed_over_the_cpus_and_named() {
    let counts = interrupts(INTERRUPTS_BEFORE);

    assert_eq!(counts.get("24 amdgpu"), Some(&300));
    assert_eq!(counts.get("LOC (Local timer interrupts)"), Some(&11000));
    assert_eq!(counts.get("0 timer"), Some(&10));
}

#[test]
fn the_top_is_what_grew_most_per_second_and_nothing_that_stood_still() {
    let top = top(&interrupts(INTERRUPTS_BEFORE), &interrupts(INTERRUPTS_AFTER), 10.0, 5);

    assert_eq!(
        top,
        [
            ("LOC (Local timer interrupts)".to_owned(), 1000.0),
            ("24 amdgpu".to_owned(), 60.0)
        ]
    );
}

#[test]
fn a_run_averages_the_draw_and_ranks_the_wakeups() {
    let root = FakeRoot::laptop();
    root.write("proc/interrupts", INTERRUPTS_BEFORE.trim_end());
    let draws = ["1000000", "1100000", "1200000"];
    let tick = Cell::new(0);
    let mut sleep = |_| {
        let n = tick.get();
        root.write("sys/class/power_supply/BAT0/current_now", draws[n.min(2)]);
        if n == 2 {
            root.write("proc/interrupts", INTERRUPTS_AFTER.trim_end());
        }
        tick.set(n + 1);
    };

    let report = run(
        &root.machine(),
        "Super Saver",
        Duration::from_secs(3),
        Duration::from_secs(1),
        &mut sleep,
    );

    assert_eq!(report.watts.len(), 3);
    assert!((report.average().expect("an average") - 17.6).abs() < 1e-9);
    assert_eq!(report.interrupts[0].0, "LOC (Local timer interrupts)");
    let text = report.text();
    assert!(
        text.starts_with("Power measurement: Super Saver for 3 s\n\nBattery draw: 17.60 W average, 16.00 to 19.20 W")
    );
}

#[test]
fn plugged_in_the_report_asks_to_unplug() {
    let root = FakeRoot::laptop();
    root.write("sys/class/power_supply/BAT0/status", "Charging");

    let report = run(
        &root.machine(),
        "Balanced",
        Duration::from_secs(1),
        Duration::from_secs(1),
        &mut |_| {},
    );

    assert!(
        report
            .text()
            .contains("Battery draw: none, it's plugged in. Unplug to measure the draw.")
    );
}

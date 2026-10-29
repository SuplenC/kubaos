.pragma library

// The system monitor's colour steps, from a sample and the thresholds in sysmon.toml: 0 fine,
// 1 close to a threshold (amber), 2 over it (red). `Sysmon` reads them, and the panel's subtitle
// reads the worst.

// Within this of its threshold a reading is close to it: CPU temperature in °C, memory in percent.
const MARGIN = 10;
// The same for a disk's percent full and the battery's draw in watts.
const NARROW_MARGIN = 5;

// The step of `value` against `threshold`; a reading the machine doesn't have is fine.
function level(value, threshold, margin) {
    if (value === null || value === undefined)
        return 0;
    return value > threshold ? 2 : value > threshold - margin ? 1 : 0;
}

function tempLevel(sample, thresholds) {
    return level(sample?.cpu?.temp, thresholds.cpuTemp, MARGIN);
}

function ramLevel(sample, thresholds) {
    return level(sample?.memory?.percent, thresholds.ram, MARGIN);
}

function diskLevel(disk, thresholds) {
    return level(disk?.percent, thresholds.disk, NARROW_MARGIN);
}

// What the battery draws counts only while it discharges: on the charger the same watts are what
// it takes, as the engine's alert reads them too.
function drawLevel(sample, thresholds) {
    const battery = sample?.battery;
    if (battery?.status !== "Discharging")
        return 0;
    return level(battery.watts, thresholds.batteryDraw, NARROW_MARGIN);
}

// The worst step of every reading that has one: CPU temperature, memory, each disk, the battery.
function worst(sample, thresholds) {
    const disks = (sample?.disks ?? []).map(disk => diskLevel(disk, thresholds));
    return Math.max(tempLevel(sample, thresholds), ramLevel(sample, thresholds), drawLevel(sample, thresholds), ...disks);
}

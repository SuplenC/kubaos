pragma Singleton

import Quickshell
import Quickshell.Services.UPower

// Laptop battery from UPower.
Singleton {
    readonly property UPowerDevice device: UPower.displayDevice
    readonly property bool available: UPower.devices.values.some(d => d.isLaptopBattery)
    readonly property real percentage: device?.percentage ?? 0
    readonly property bool charging: device?.state === UPowerDeviceState.Charging || device?.state === UPowerDeviceState.FullyCharged || device?.state === UPowerDeviceState.PendingCharge
    readonly property bool low: !charging && percentage < 0.15
    readonly property string icon: {
        if (charging)
            return "battery_charging_full";
        if (percentage > 0.95)
            return "battery_full";
        // KubaOS Symbols draw the battery in four levels short of full.
        return `battery_${Math.min(3, Math.floor(percentage * 4))}_bar`;
    }
    readonly property string timeLeft: {
        const seconds = charging ? device?.timeToFull : device?.timeToEmpty;
        if (!seconds)
            return "";
        const h = Math.floor(seconds / 3600);
        const m = Math.round((seconds % 3600) / 60);
        return `${h}h ${m}m ${charging ? "to full" : "left"}`;
    }
}

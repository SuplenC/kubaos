pragma Singleton

import Quickshell
import Quickshell.Services.UPower

// power-profiles-daemon over D-Bus.
Singleton {
    id: root

    readonly property int current: PowerProfiles.profile
    readonly property list<int> available: PowerProfiles.hasPerformanceProfile ? [PowerProfile.PowerSaver, PowerProfile.Balanced, PowerProfile.Performance] : [PowerProfile.PowerSaver, PowerProfile.Balanced]
    readonly property string icon: iconFor(current)

    function set(profile: int): void {
        PowerProfiles.profile = profile;
    }

    function cycle(): void {
        const i = available.indexOf(current);
        set(available[(i + 1) % available.length]);
    }

    function iconFor(profile: int): string {
        return profile === PowerProfile.PowerSaver ? "eco" : profile === PowerProfile.Performance ? "bolt" : "balance";
    }

    function nameFor(profile: int): string {
        return profile === PowerProfile.PowerSaver ? "Power saver" : profile === PowerProfile.Performance ? "Performance" : "Balanced";
    }
}

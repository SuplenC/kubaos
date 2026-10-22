use super::*;

/// `wpctl status` from PipeWire 1.4, with a second sink and a Video section
/// whose lists share the headings.
const STATUS: &str = "\
PipeWire 'pipewire-0' [1.4.7, kuba@tuxedo, cookie:3550361094]
 └─ Clients:
        33. WirePlumber                         [1.4.7, kuba@tuxedo, pid:1673]
        34. pipewire                            [1.4.7, kuba@tuxedo, pid:1672]

Audio
 ├─ Devices:
 │      46. Built-in Audio                      [alsa]
 │
 ├─ Sinks:
 │      52. Built-in Audio HDMI / DisplayPort 1 [vol: 1.00]
 │  *   53. Built-in Audio Analog Stereo        [vol: 0.45 MUTED]
 │
 ├─ Sink endpoints:
 │
 ├─ Sources:
 │  *   54. Built-in Audio Digital Microphone   [vol: 1.00]
 │
 ├─ Source endpoints:
 │
 ├─ Filters:
 │
 └─ Streams:
        80. Firefox
             81. output_FL       > Built-in Audio:playback_FL	[active]

Video
 ├─ Devices:
 │      49. Integrated Camera                   [v4l2]
 │
 ├─ Sinks:
 │
 ├─ Sources:
 │  *   57. Integrated Camera (V4L2)
 │
 └─ Streams:

Settings
 └─ Default Configured Node Names:
         0. Audio/Sink    alsa_output.pci-0000_00_1f.3.analog-stereo
";

#[test]
fn the_outputs_are_the_audio_sinks_with_the_default_marked() {
    let outputs = devices(STATUS, Direction::Output);

    assert_eq!(
        outputs,
        [
            Device {
                id: 52,
                name: "Built-in Audio HDMI / DisplayPort 1".into(),
                default: false
            },
            Device {
                id: 53,
                name: "Built-in Audio Analog Stereo".into(),
                default: true
            },
        ]
    );
}

#[test]
fn the_inputs_are_the_audio_sources_never_the_cameras() {
    let inputs = devices(STATUS, Direction::Input);

    assert_eq!(
        inputs,
        [Device {
            id: 54,
            name: "Built-in Audio Digital Microphone".into(),
            default: true
        }]
    );
}

#[test]
fn no_pipewire_lists_no_devices() {
    assert!(devices("", Direction::Output).is_empty());
}

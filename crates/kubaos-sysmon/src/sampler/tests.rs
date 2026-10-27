use super::Sampler;
use crate::fixture::Machine;

#[test]
fn the_first_sample_has_no_speeds_and_the_second_does() {
    let machine = Machine::laptop();
    let mut sampler = Sampler::new(machine.root());

    let first = sampler.sample(10_000, false, false);
    machine.stat(2000, 1300);
    machine.net(2_000_000, 500_000);
    let second = sampler.sample(12_000, false, false);

    assert_eq!(first.cpu.usage, None);
    assert_eq!(first.net.down, None);
    let usage = second.cpu.usage.expect("a usage");
    assert!((usage - 50.0).abs() < 0.01, "{usage}");
    assert_eq!(second.net.interface.as_deref(), Some("wlan0"));
    assert_eq!(second.net.down, Some(1_000_000.0));
    assert_eq!(second.net.up, Some(250_000.0));
}

#[test]
fn a_light_sample_has_what_the_pill_and_the_alerts_need() {
    let machine = Machine::laptop();
    let mut sampler = Sampler::new(machine.root());

    let sample = sampler.sample(1000, false, false);

    assert!(!sample.detail);
    assert_eq!(sample.cpu.count, 2);
    assert_eq!(sample.cpu.temp, Some(55.0));
    assert_eq!(sample.cpu.load, Some([0.5, 0.4, 0.3]));
    assert!(sample.memory.is_some_and(|memory| memory.percent > 93.0));
    assert_eq!(sample.gpus[0].usage, Some(12.0));
    assert_eq!(sample.battery.as_ref().and_then(|battery| battery.watts), Some(10.0));
    assert_eq!(sample.uptime, 3725);
    assert_eq!(sample.kernel, "6.17.0-5-generic");
    // The popup's extras are left out.
    assert_eq!(sample.cpu.cores, [] as [f32; 0]);
    assert_eq!(sample.cpu.mhz, None);
    assert_eq!(sample.fans, [] as [crate::hwmon::Fan; 0]);
    assert_eq!(sample.net.address, None);
    assert!(sample.top.is_none());
}

#[test]
fn a_detailed_sample_adds_cores_clock_fans_address_and_disk_traffic() {
    let machine = Machine::laptop();
    let mut sampler = Sampler::new(machine.root());
    sampler.sample(1000, true, false);
    machine.stat(2000, 1300);
    machine.diskstats(2048, 4096);

    let sample = sampler.sample(2000, true, false);

    assert_eq!(sample.cpu.cores.len(), 2);
    assert_eq!(sample.cpu.mhz, Some(3000));
    assert_eq!(sample.fans[0].rpm, 2100);
    assert_eq!(sample.net.address.as_deref(), Some("192.168.1.42"));
    let io = sample.io.expect("disk traffic");
    assert_eq!((io.read, io.write), (2048.0 * 512.0, 4096.0 * 512.0));
}

#[test]
fn a_process_scan_fills_the_top_lists() {
    let machine = Machine::laptop();
    machine.process(100, "firefox", 100, 1000);
    machine.process(200, "chrome", 100, 3000);
    machine.process(201, "chrome", 100, 3000);
    let mut sampler = Sampler::new(machine.root());
    sampler.sample(1000, true, true);
    machine.process(100, "firefox", 400, 1000);
    machine.stat(2000, 1300);

    let sample = sampler.sample(2000, true, true);

    let top = sample.top.expect("the top lists");
    assert_eq!(top.cpu[0].name, "firefox");
    assert!((top.cpu[0].cpu - 30.0).abs() < 0.01);
    assert_eq!(top.memory[0].name, "chrome");
    let (name, bytes) = top.heaviest.expect("the heaviest");
    assert_eq!(name, "chrome");
    assert_eq!(bytes, 6000 * sampler.page());
}

#[test]
fn after_a_restart_the_next_sample_counts_from_the_restart() {
    let machine = Machine::laptop();
    let mut sampler = Sampler::new(machine.root());
    sampler.sample(1000, false, false);
    // Busy for the whole pause.
    machine.stat(100_000, 800);
    sampler.restart(60_000, false);
    machine.stat(101_000, 1700);

    let sample = sampler.sample(61_000, false, false);

    let usage = sample.cpu.usage.expect("a usage");
    assert!((usage - 10.0).abs() < 0.01, "{usage}");
}

#[test]
fn a_machine_with_nothing_readable_still_samples() {
    let empty = tempfile::tempdir().expect("a temp dir");
    let mut sampler = Sampler::new(empty.path());

    let sample = sampler.sample(1000, true, true);

    assert_eq!(sample.cpu.count, 0);
    assert!(sample.memory.is_none());
    assert_eq!(sample.gpus, [] as [crate::gpu::Gpu; 0]);
    assert!(sample.battery.is_none());
    assert_eq!(sample.net.interface, None);
    assert!(sample.top.is_some_and(|top| top.cpu.is_empty()));
}

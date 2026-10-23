use super::Available;
use super::parse_notified;
use crate::update::policy::Target;

fn target(commit: &str, label: &str, codename: Option<&str>) -> Target {
    Target {
        commit: commit.into(),
        version: None,
        codename: codename.map(Into::into),
        label: label.into(),
    }
}

#[test]
fn the_file_names_the_update_and_the_upgrade() {
    let available = Available {
        update: Some(target("aaa", "0.1.2", None)),
        upgrade: Some(target("bbb", "0.2.0", Some("akihabara"))),
    };

    assert_eq!(
        available.to_text(),
        "update=0.1.2\nupdate-commit=aaa\nupgrade=0.2.0\nupgrade-commit=bbb\nupgrade-codename=akihabara\n"
    );
}

#[test]
fn each_notification_is_sent_once() {
    let available = Available {
        update: Some(target("aaa", "main at aaa", None)),
        upgrade: Some(target("bbb", "0.2.0", Some("akihabara"))),
    };

    let first = available.notifications(&parse_notified(""));
    let again = available.notifications(&parse_notified("update aaa\n"));

    assert_eq!(
        first,
        [
            (
                "update aaa".to_owned(),
                "KubaOS main at aaa is ready. Update when you feel like it: kubaos update. Or don't, I'm not your mum."
                    .to_owned()
            ),
            (
                "upgrade bbb".to_owned(),
                "KubaOS Akihabara (0.2.0) is out. kubaos upgrade when you're ready, I'll wait.".to_owned()
            ),
        ]
    );
    assert_eq!(again.len(), 1);
    assert_eq!(again[0].0, "upgrade bbb");
}

#[test]
fn an_update_notification_names_the_release() {
    let available = Available {
        update: Some(target("ccc", "0.1.3", Some("shinjuku"))),
        upgrade: None,
    };

    let sent = available.notifications(&parse_notified(""));

    assert_eq!(
        sent[0].1,
        "Shinjuku 0.1.3 is ready. Update when you feel like it: kubaos update. Or don't, I'm not your mum."
    );
}

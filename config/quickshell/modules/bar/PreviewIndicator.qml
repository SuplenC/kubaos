import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services

// While the home is a live preview of a KubaOS checkout: a cold pill naming its
// branch, as a cold chip marks a preview. The hint gives the commit and the way
// back; a click opens the Command menu at "Back to the release".
Pill {
    id: root

    visible: Preview.active
    interactive: true
    spacing: K.Tokens.space.s4
    color: K.Look.alpha(K.Tokens.color.cold, K.Tokens.opacity.tint)
    ink: K.Tokens.color.cold
    tooltip: {
        const at = Preview.dirty ? `${Preview.commit} with uncommitted changes` : Preview.commit;
        const since = Preview.since > 0 ? `, since ${Qt.formatDateTime(new Date(Preview.since), Time.timeFormat)}` : "";
        const of = Preview.branch !== "" ? `${Preview.branch} at ${at}` : at;
        return `Live preview of ${of}${since}. Click to go back to the release`;
    }
    onClicked: MenuRequests.open("Back to the release")

    K.Icon {
        name: "code"
        size: K.Tokens.type.label.icon
        color: K.Tokens.color.cold
    }

    // The branch's last part, `live-preview` of `feature/live-preview`: the hint has it whole.
    K.Text {
        text: Preview.name.slice(Preview.name.lastIndexOf("/") + 1)
        style: K.Tokens.type.label
        color: K.Tokens.color.cold
        room: K.Tokens.size.bar_label
        maximumLines: 1
    }
}

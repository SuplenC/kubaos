pragma Singleton
import QtQuick

QtObject {
    readonly property string home: "/home/kuba"
    readonly property string layer: "/home/kuba/.config/kubaos"
    readonly property var apps: ({})

    function dir(app: string): string {
        return `/home/kuba/.config/kubaos/${app}`;
    }

    function shown(app: string, name: string): string {
        return `~/.config/kubaos/desktop/${app}/${name}`;
    }
}

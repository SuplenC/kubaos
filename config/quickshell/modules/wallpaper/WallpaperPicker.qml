import QtQuick
import QtQuick.Layouts
import Quickshell.Widgets
import qs.theme
import qs.components
import qs.services

// Wallpaper picker, rising out of the bottom edge: one tab per folder
// (Wallpapers.folders: live, the chosen sets, yours), a grid
// of 16:10 thumbnails. Moving the selection previews it on the desktop; Enter
// applies it through `dot-theme set`, which also regenerates all colors.
// Moving wallpapers carry a badge and play in their thumbnail while selected
// or hovered (unless motion is set to never). ukiyo's live scenes are the
// first folder, with a "live" badge and their poster. The last tab is the
// wallpaper settings (WallpaperSettings.qml): Motion and every ukiyo option.
// Keys: hjkl/arrows move, Tab/Shift+Tab switch folder, S settings, Enter
// apply, Esc close (from the settings: back to the wallpapers).
Drawer {
    id: root

    property int folderIndex: 0
    // The tab before the settings, for Esc to go back to.
    property int lastFolder: 0
    readonly property var folders: Wallpapers.folders
    readonly property int settingsIndex: folders.length
    readonly property bool settingsOpen: folderIndex === settingsIndex
    readonly property string folder: folders[settingsOpen ? lastFolder : folderIndex]?.name ?? ""
    readonly property var images: Wallpapers.images[folder] ?? []
    readonly property real dpr: frame.screen?.devicePixelRatio ?? 1
    // Up to four per row; fewer images make a narrower panel (it morphs when
    // switching folders).
    readonly property int columns: Math.max(2, Math.min(4, images.length))
    readonly property real thumbWidth: 248
    readonly property real thumbHeight: Math.round(thumbWidth * 10 / 16)
    readonly property real cellWidth: thumbWidth + Theme.spacing.md
    readonly property real cellHeight: thumbHeight + 34
    readonly property int rows: Math.max(1, Math.min(2, Math.ceil(images.length / columns)))

    edge: "bottom"
    contentWidth: (settingsOpen ? 4 : columns) * cellWidth + Theme.spacing.xl * 2 - Theme.spacing.md
    contentHeight: header.implicitHeight + (settingsOpen ? Math.min(560, (frame.b - frame.t) * 0.7) : rows * cellHeight) + Theme.spacing.lg * 2 + Theme.spacing.md
    onOpened: {
        Wallpapers.refresh();
        const current = Wallpapers.folderOf(Wallpapers.current);
        folderIndex = Math.max(0, folders.findIndex(f => f.name === current));
        lastFolder = folderIndex;
        if (Wallpapers.openSettings) {
            Wallpapers.openSettings = false;
            pick(settingsIndex);
            return;
        }
        selectCurrent();
        grid.forceActiveFocus();
    }
    onShownChanged: if (!shown)
        Wallpapers.preview = ""
    // After the grid has taken the new model (a model reset puts it back at 0).
    onImagesChanged: Qt.callLater(selectCurrent)

    function selectCurrent(): void {
        grid.currentIndex = Math.max(0, images.indexOf(Wallpapers.current));
    }

    // Folders and the settings tab, in a ring.
    function switchFolder(delta: int): void {
        const n = folders.length + 1;
        pick((folderIndex + delta + n) % n);
    }

    function pick(index: int): void {
        folderIndex = index;
        if (settingsOpen) {
            Wallpapers.preview = "";
            Ukiyo.refreshSchema();
            settings.forceActiveFocus();
        } else {
            lastFolder = index;
            grid.forceActiveFocus();
        }
    }

    function apply(index: int): void {
        grid.currentIndex = index;
        Wallpapers.set(images[index]);
    }

    // Preview the selection on the desktop after a short pause.
    Timer {
        id: previewDelay

        interval: 180
        onTriggered: {
            const path = root.images[grid.currentIndex] ?? "";
            Wallpapers.preview = root.shown && path !== Wallpapers.current ? path : "";
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.leftMargin: Theme.spacing.xl
        anchors.rightMargin: Theme.spacing.xl
        anchors.topMargin: Theme.spacing.lg
        anchors.bottomMargin: Theme.spacing.lg
        spacing: Theme.spacing.md

        RowLayout {
            id: header

            Layout.fillWidth: true
            spacing: Theme.spacing.sm

            TabBar {
                labels: [...root.folders.map(f => `${f.name.charAt(0).toUpperCase()}${f.name.slice(1).replace(/-/g, " ")}  ${(Wallpapers.images[f.name] ?? []).length}`), "Settings"]
                currentIndex: root.folderIndex
                activeFocusOnTab: false
                onPicked: index => root.pick(index)
            }

            Item {
                Layout.fillWidth: true
            }

            Repeater {
                model: root.settingsOpen ? [["Tab", "next"], ["←→", "change"], ["Space", "toggle"], ["Del", "reset"], ["Esc", "back"]] : [["Tab", "folder"], ["S", "settings"], ["Enter", "apply"], ["Esc", "close"]]

                delegate: Row {
                    required property var modelData

                    spacing: Theme.spacing.xs

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        implicitWidth: key.implicitWidth + Theme.spacing.sm * 2
                        implicitHeight: 20
                        radius: Theme.rounding.xs
                        color: Theme.m3surfaceContainerHigh

                        StyledText {
                            id: key

                            anchors.centerIn: parent
                            text: modelData[0]
                            font.pixelSize: Theme.font.xs
                            font.weight: Font.DemiBold
                            color: Theme.m3onSurfaceVariant
                        }
                    }

                    StyledText {
                        anchors.verticalCenter: parent.verticalCenter
                        rightPadding: Theme.spacing.sm
                        text: modelData[1]
                        font.pixelSize: Theme.font.sm
                        color: Theme.m3outline
                    }
                }
            }
        }

        WallpaperSettings {
            id: settings

            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: root.settingsOpen
            onBack: root.pick(root.lastFolder)
        }

        GridView {
            id: grid

            // Exactly `columns` cells wide; each cell carries its own right gap.
            Layout.preferredWidth: root.columns * root.cellWidth
            Layout.fillHeight: true
            visible: !root.settingsOpen
            clip: true
            cellWidth: root.cellWidth
            cellHeight: root.cellHeight
            boundsBehavior: Flickable.StopAtBounds
            highlightMoveDuration: Theme.anim.normal
            highlightFollowsCurrentItem: true
            keyNavigationWraps: true
            model: root.images
            onCurrentIndexChanged: if (root.shown)
                previewDelay.restart()

            Keys.onPressed: event => {
                const k = event.key;
                if (k === Qt.Key_H)
                    moveCurrentIndexLeft();
                else if (k === Qt.Key_L)
                    moveCurrentIndexRight();
                else if (k === Qt.Key_J)
                    moveCurrentIndexDown();
                else if (k === Qt.Key_K)
                    moveCurrentIndexUp();
                else if (k === Qt.Key_Tab)
                    root.switchFolder(1);
                else if (k === Qt.Key_S)
                    root.pick(root.settingsIndex);
                else if (k === Qt.Key_Backtab)
                    root.switchFolder(-1);
                else if ((k === Qt.Key_Return || k === Qt.Key_Enter) && count > 0)
                    root.apply(currentIndex);
                else
                    return;
                event.accepted = true;
            }

            delegate: Item {
                id: cell

                required property string modelData
                required property int index
                readonly property bool isCurrent: GridView.isCurrentItem
                readonly property bool applied: modelData === Wallpapers.current

                width: grid.cellWidth
                height: grid.cellHeight

                Rectangle {
                    id: frameRect

                    width: root.thumbWidth
                    height: root.thumbHeight
                    radius: Theme.rounding.lg + 3
                    color: "transparent"
                    border.width: 2
                    border.color: cell.isCurrent ? Theme.m3primary : "transparent"
                    scale: cell.isCurrent ? 1 : 0.94

                    Behavior on scale {
                        Anim {
                            duration: Theme.anim.large
                            easing.bezierCurve: Theme.anim.spatial
                        }
                    }

                    Behavior on border.color {
                        CAnim {}
                    }

                    ClippingRectangle {
                        anchors.fill: parent
                        anchors.margins: 4
                        radius: Theme.rounding.lg
                        color: Theme.m3surfaceContainerHigh

                        WallpaperView {
                            id: thumb

                            anchors.fill: parent
                            path: cell.modelData
                            // Decode at the size it is shown, in device pixels: crisp and cheap.
                            sourceSize: Qt.size(Math.ceil(width * root.dpr), Math.ceil(height * root.dpr))
                            playing: root.shown && (cell.isCurrent || tap.containsMouse) && Motion.mode !== "never"
                            opacity: ready ? (cell.isCurrent ? 1 : 0.7) : 0

                            Behavior on opacity {
                                Anim {
                                    duration: Theme.anim.normal
                                    easing.bezierCurve: Theme.anim.effects
                                }
                            }
                        }

                        // Live badge: a ukiyo scene, it reacts to what you do.
                        Rectangle {
                            anchors.left: parent.left
                            anchors.bottom: parent.bottom
                            anchors.margins: Theme.spacing.sm
                            width: liveRow.implicitWidth + Theme.spacing.md * 2 - 2
                            height: 26
                            radius: 13
                            color: Theme.m3primaryContainer
                            visible: Wallpapers.isScene(cell.modelData)

                            Row {
                                id: liveRow

                                anchors.centerIn: parent
                                spacing: Theme.spacing.xs

                                Rectangle {
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 7
                                    height: 7
                                    radius: 4
                                    color: Theme.m3onPrimaryContainer
                                    opacity: Ukiyo.current === cell.modelData.slice(6) && Ukiyo.shown ? 1 : 0.55
                                }

                                StyledText {
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: "live"
                                    font.pixelSize: Theme.font.sm
                                    font.weight: Font.DemiBold
                                    font.letterSpacing: 0.5
                                    color: Theme.m3onPrimaryContainer
                                }
                            }
                        }

                        // Motion badge: this one moves.
                        Rectangle {
                            anchors.left: parent.left
                            anchors.bottom: parent.bottom
                            anchors.margins: Theme.spacing.sm
                            width: 26
                            height: 26
                            radius: 13
                            color: Theme.alpha(Theme.m3surfaceContainerHighest, 0.85)
                            visible: thumb.video

                            Icon {
                                anchors.centerIn: parent
                                name: thumb.moving ? "motion_photos_on" : "motion_photos_paused"
                                size: 18
                                color: Theme.m3primary
                            }
                        }

                        Rectangle {
                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.margins: Theme.spacing.sm
                            width: 26
                            height: 26
                            radius: 13
                            color: Theme.m3primary
                            visible: cell.applied

                            Icon {
                                anchors.centerIn: parent
                                name: "check"
                                size: 18
                                color: Theme.m3onPrimary
                            }
                        }

                        StateLayer {
                            id: tap

                            color: Theme.m3onSurface
                            onClicked: root.apply(cell.index)
                        }
                    }
                }

                StyledText {
                    anchors.top: frameRect.bottom
                    anchors.topMargin: Theme.spacing.xs
                    anchors.horizontalCenter: frameRect.horizontalCenter
                    width: root.thumbWidth - Theme.spacing.md
                    horizontalAlignment: Text.AlignHCenter
                    text: Wallpapers.label(cell.modelData)
                    color: cell.isCurrent ? Theme.m3onSurface : Theme.m3onSurfaceVariant
                    font.pixelSize: Theme.font.sm
                    font.weight: cell.isCurrent ? Font.Medium : Font.Normal
                }
            }

            StyledText {
                anchors.centerIn: parent
                visible: grid.count === 0
                text: `Nothing in ${root.folders[root.lastFolder]?.dir || root.folder}. Drop some images or videos in.`
                color: Theme.m3onSurfaceVariant
            }
        }
    }
}

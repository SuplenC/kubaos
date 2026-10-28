import QtQuick
import Quickshell.Widgets
import KubaOS.Components as K
import qs.components
import qs.services

// Wallpaper picker, rising out of the bottom edge: one tab per folder
// (Wallpapers.folders: the chosen sets, yours), a grid
// of 16:10 thumbnails. Moving the selection previews it on the desktop; Enter
// applies it through `dot-theme set`, which also regenerates all colors.
// Moving wallpapers carry a badge and play in their thumbnail while selected
// or hovered (unless motion is set to never). The last tab is the
// wallpaper settings (WallpaperSettings.qml).
// Keys: hjkl/arrows move, Tab/Shift+Tab switch folder, S settings, Enter
// apply, Esc close (from the settings: back to the wallpapers).
K.Sheet {
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
    // Above and below the header and the thumbnails.
    readonly property int pad: K.Tokens.space.s24
    // At the sides, the container padding round the thumbnails or the tabs and key hints,
    // whichever is wider; the screen's width's own when they would take more than it has.
    readonly property int bodyInset: {
        const inner = Math.max(gridWidth, headerWidth);
        const around = K.Tokens.containerPaddingAround(inner);
        return inner + around * 2 > widest ? K.Tokens.containerPadding(widest) : around;
    }
    // The widest the panel grows: the screen between its frame's corners.
    readonly property real widest: frame.r - frame.l - K.Tokens.radius.surface * 2
    // Up to four per row; fewer images make a narrower panel (it morphs when
    // switching folders).
    readonly property int mostColumns: 4
    readonly property int columns: Math.max(2, Math.min(mostColumns, images.length))
    // The most of the screen's height the settings take before they scroll.
    readonly property real settingsShare: 0.7
    // 16:10 thumbnails, 256 by 160: both on the grid, so their edges land on device pixels at
    // every scale.
    readonly property real thumbWidth: K.Tokens.size.thumbnail_width
    readonly property real thumbHeight: K.Tokens.size.thumbnail_height
    // The focus ring's band round each thumbnail, inside its cell so the grid's clip never cuts it.
    readonly property real ring: K.Tokens.focus.offset
    readonly property real cellWidth: thumbWidth + ring * 2
    readonly property real cellHeight: ring + thumbHeight + K.Tokens.space.s8 + K.Tokens.type.small.line + K.Tokens.space.s8
    readonly property int rows: Math.max(1, Math.min(2, Math.ceil(images.length / columns)))
    // The thumbnails, from the first's left edge to the last's right.
    readonly property real gridWidth: (settingsOpen ? mostColumns : columns) * cellWidth - ring * 2
    readonly property real headerWidth: tabs.implicitWidth + K.Tokens.space.s16 * 2 + hints.implicitWidth

    edge: "bottom"
    // As wide as the thumbnails or the tabs and key hints above them,
    // whichever is wider, and never wider than the screen.
    contentWidth: Math.min(Math.max(gridWidth, headerWidth) + bodyInset * 2, widest)
    contentHeight: pad + header.implicitHeight + K.Tokens.space.s16 + (settingsOpen ? Math.min(Math.max(cellHeight, settings.implicitHeight), (frame.b - frame.t) * settingsShare) : rows * cellHeight) + pad
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

    // The folders' tabs, and the keys as key hints at the far end, as every screen shows them.
    K.Row {
        id: header

        x: root.bodyInset
        y: root.pad
        width: root.contentWidth - root.bodyInset * 2
        gap: K.Tokens.space.s16
        align: "centre"

        // Picked with the pointer; Tab and Shift+Tab in the grid switch folders.
        K.Tabs {
            id: tabs

            labels: [...root.folders.map(f => `${f.name.charAt(0).toUpperCase()}${f.name.slice(1).replace(/-/g, " ")} ${(Wallpapers.images[f.name] ?? []).length}`), "Settings"]
            currentIndex: root.folderIndex
            activeFocusOnTab: false
            onPicked: index => root.pick(index)
        }

        K.Fill {
        }

        K.KeyHints {
            id: hints

            hints: root.settingsOpen ? [["← →", "change"], ["Esc", "back"]] : [["Tab", "folder"], ["S", "settings"], ["Enter", "apply"], ["Esc", "close"]]
        }
    }

    WallpaperSettings {
        id: settings

        x: root.bodyInset
        y: header.y + header.height + K.Tokens.space.s16
        width: root.contentWidth - root.bodyInset * 2
        height: root.contentHeight - y - root.pad
        visible: root.settingsOpen
        onBack: root.pick(root.lastFolder)
    }

    GridView {
        id: grid

        // Exactly `columns` cells wide, centred, so the thumbnails sit in the middle when the
        // header makes the panel wider than they are.
        x: K.Look.snap((root.contentWidth - width) / 2, root.dpr)
        y: header.y + header.height + K.Tokens.space.s16 - root.ring
        width: root.columns * root.cellWidth
        height: root.rows * root.cellHeight
        visible: !root.settingsOpen
        // Cut at its own whole rows, the focus ring's band included, and scrolled a row at a time, so
        // no tile is ever cut through.
        clip: true
        snapMode: GridView.SnapToRow
        cellWidth: root.cellWidth
        cellHeight: root.cellHeight
        boundsBehavior: Flickable.StopAtBounds
        highlightMoveDuration: K.Tokens.duration.medium
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

            // A picture to pick, the whole tile: the applied one ringed and marked Current, the
            // keyboard's the ring (cold on the applied one), the name under it. Its box is the
            // wallpaper's aspect from the first frame, the placeholder in it until the picture or
            // the video's poster fades in over it.
            K.PictureChoice {
                id: picture

                x: root.ring
                y: root.ring
                width: root.thumbWidth
                height: implicitHeight
                aspect: K.Tokens.aspect.wallpaper
                name: Wallpapers.label(cell.modelData)
                picked: cell.applied
                markIcon: "check"
                markText: "Current"
                focused: cell.isCurrent
                // This one moves: playing, or held on its poster.
                badgeIcon: thumb.moving ? "motion_photos_on" : "motion_photos_paused"
                badgeText: thumb.video ? (thumb.moving ? "Playing" : "Video") : ""
                onActivated: root.apply(cell.index)

                ClippingRectangle {
                    anchors.fill: parent
                    radius: K.Tokens.radius.control
                    color: "transparent"

                    WallpaperView {
                        id: thumb

                        anchors.fill: parent
                        path: cell.modelData
                        // Decode at the size it is shown, in device pixels: crisp and cheap.
                        sourceSize: Qt.size(Math.ceil(width * root.dpr), Math.ceil(height * root.dpr))
                        playing: root.shown && (cell.isCurrent || picture.hovered) && Motion.mode !== "never"
                        // The ones not under the keyboard sit back, as secondary content does.
                        opacity: ready ? (cell.isCurrent ? 1 : K.Tokens.opacity.subdued) : 0

                        Behavior on opacity {
                            NumberAnimation {
                                duration: K.Tokens.duration.short
                                easing.type: Easing.BezierSpline
                                easing.bezierCurve: K.Tokens.easing.standard
                            }
                        }
                    }
                }
            }
        }

        K.EmptyState {
            anchors.centerIn: parent
            width: parent.width
            visible: grid.count === 0
            icon: "wallpaper"
            title: "Nothing here yet"
            body: `Drop some images or videos in ${root.folders[root.folderIndex]?.dir || root.folder}.`
        }
    }
}

import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.theme
import qs.settings
import qs.components
import qs.services
import "fuzzy.js" as Fuzzy

// App launcher with fuzzy search, dropping out of the bar under the clock.
// Input starting with "=" or made of numbers and operators goes to the
// calculator instead; Enter copies the result.
// Keys: type to search, Up/Down or Ctrl+j/k/n/p to move, Enter to launch, Esc to close.
Drawer {
    id: root

    readonly property string query: search.text.trim()
    readonly property bool calcMode: query.startsWith("=") || (/^[\d\s.,+\-*\/^%()]+$/.test(query) && /\d\s*[+\-*\/^%]/.test(query))
    // NoDisplay and Hidden entries are already left out by Quickshell.
    readonly property var entries: DesktopEntries.applications.values.filter(e => !Settings.launcherHidden.includes(e.id))
    readonly property var apps: calcMode ? [] : Fuzzy.rank(entries, query, nameOf)
    readonly property int rowHeight: 46
    readonly property int maxRows: 7

    edge: "top"
    contentWidth: 560
    contentHeight: column.implicitHeight + Theme.spacing.lg * 2
    onOpened: {
        search.text = "";
        list.currentIndex = 0;
        search.focusInput();
        list.playEnter();
    }

    function nameOf(entry: DesktopEntry): string {
        return Settings.launcherNames[entry.id] ?? entry.name;
    }

    function launch(entry: DesktopEntry): void {
        if (entry.runInTerminal)
            Quickshell.execDetached({
                command: [...Settings.terminal, ...entry.command],
                workingDirectory: entry.workingDirectory
            });
        else
            entry.execute();
        Panels.close();
    }

    function copyResult(): void {
        if (!calculator.result)
            return;
        Quickshell.clipboardText = calculator.result;
        Panels.close();
    }

    Calculator {
        id: calculator

        expression: root.calcMode ? root.query.replace(/^=/, "") : ""
    }

    ColumnLayout {
        id: column

        x: Theme.spacing.lg
        y: Theme.spacing.lg
        width: root.contentWidth - Theme.spacing.lg * 2
        spacing: Theme.spacing.md

        SearchField {
            id: search

            Layout.fillWidth: true
            placeholder: "Search apps, or = to calculate"
            icon: root.calcMode ? "calculate" : "search"
            iconColor: root.calcMode ? Theme.m3tertiary : Theme.m3onSurfaceVariant
            forwardTo: [list]
            onAccepted: if (root.calcMode)
                root.copyResult()
            onTextChanged: list.currentIndex = 0
        }

        // Calculator result card.
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: root.calcMode ? 76 : 0
            radius: Theme.rounding.lg
            color: Theme.m3tertiaryContainer
            opacity: root.calcMode ? 1 : 0
            visible: implicitHeight > 0.5
            clip: true

            Behavior on implicitHeight {
                Anim {
                    duration: Theme.anim.large
                    easing.bezierCurve: Theme.anim.spatial
                }
            }

            Behavior on opacity {
                Anim {
                    duration: Theme.anim.small
                }
            }

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.spacing.lg
                anchors.rightMargin: Theme.spacing.lg
                spacing: Theme.spacing.md

                Icon {
                    name: "equal"
                    size: 28
                    color: Theme.m3onTertiaryContainer
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 0

                    StyledText {
                        Layout.fillWidth: true
                        text: calculator.result || "…"
                        color: Theme.m3onTertiaryContainer
                        font.pixelSize: Theme.font.xxl
                        font.weight: Font.Medium
                        font.features: Theme.font.tabular
                    }

                    StyledText {
                        Layout.fillWidth: true
                        text: root.query.replace(/^=/, "").trim()
                        color: Theme.alpha(Theme.m3onTertiaryContainer, 0.7)
                        font.pixelSize: Theme.font.sm
                    }
                }

                Rectangle {
                    implicitWidth: hint.implicitWidth + Theme.spacing.md * 2
                    implicitHeight: 26
                    radius: Theme.rounding.full
                    color: Theme.alpha(Theme.m3onTertiaryContainer, 0.12)

                    StyledText {
                        id: hint

                        anchors.centerIn: parent
                        text: "Enter to copy"
                        color: Theme.m3onTertiaryContainer
                        font.pixelSize: Theme.font.sm
                        font.weight: Font.Medium
                    }
                }
            }

            StateLayer {
                color: Theme.m3onTertiaryContainer
                onClicked: root.copyResult()
            }
        }

        NavList {
            id: list

            Layout.fillWidth: true
            Layout.preferredHeight: root.calcMode ? 0 : Math.min(count, root.maxRows) * (root.rowHeight + spacing)
            visible: Layout.preferredHeight > 0
            vimKeys: false
            model: ScriptModel {
                values: root.apps
            }
            onActivated: index => root.launch(root.apps[index])

            Behavior on Layout.preferredHeight {
                Anim {
                    duration: Theme.anim.normal
                    easing.bezierCurve: Theme.anim.emphasizedDecel
                }
            }

            delegate: ListRow {
                required property DesktopEntry modelData
                required property int index

                width: ListView.view.width
                implicitHeight: root.rowHeight
                enterIndex: index
                // modelData is briefly null while DesktopEntries reloads and
                // the delegate is torn down.
                image: modelData ? Quickshell.iconPath(modelData.icon, "application-x-executable") : ""
                title: modelData ? root.nameOf(modelData) : ""
                subtitle: modelData ? (modelData.genericName || modelData.comment) : ""
                current: ListView.isCurrentItem
                onClicked: root.launch(modelData)
            }
        }

        StyledText {
            Layout.alignment: Qt.AlignHCenter
            Layout.margins: Theme.spacing.md
            visible: !root.calcMode && list.count === 0
            text: "No app by that name. Typo?"
            color: Theme.m3onSurfaceVariant
        }
    }
}

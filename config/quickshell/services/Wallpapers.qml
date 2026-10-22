pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import qs.theme

// Current wallpaper (from ~/.local/state/theme/wallpaper, written by dot-theme)
// and the wallpapers available in each folder: stills, and videos that loop
// over a poster frame (<name>.poster.jpg next to <name>.mp4; dot-theme makes
// one in ~/.local/state/theme/posters/ for a video that has none). The
// folders are `dot-theme folders`: the chosen sets in
// /usr/share/kubaos/wallpapers, then ~/Pictures/wallpapers.
Singleton {
    id: root

    readonly property string current: currentFile.text().trim() || (Theme.palette.wallpaper ?? "")
    // Shown on the desktop instead of `current` while the picker browses
    // (colors only change once a wallpaper is applied). "" = no preview.
    property string preview: ""
    readonly property string shown: preview || current
    // The picker opens on its settings tab next time (`qs ipc call wallpaper settings`).
    property bool openSettings: false
    // The picker's tabs, in order: [{ name, dir }].
    // dot-theme decides which sets show (scripts/dot-theme, `folders`).
    property var folders: []
    // folder name -> the absolute paths it shows, sorted (`dot-theme wallpapers`)
    property var images: ({})
    // Poster files next to videos and in dot-theme's cache, as a set (path -> true).
    property var posters: ({})
    // The first listing is in: `poster()` knows what exists.
    property bool listed: false
    readonly property list<string> videoExtensions: ["mp4", "webm", "mkv", "mov"]
    readonly property list<string> imageExtensions: ["jpg", "jpeg", "png", "webp"]

    function isVideo(path: string): bool {
        return videoExtensions.includes(path.split(".").pop().toLowerCase());
    }

    // The still that stands in for a video: the picker thumbnail, what shows
    // before the first frame and whenever motion is off. "" when it has none
    // (or before the first listing). Must match poster_for in scripts/dot-theme.
    function poster(path: string): string {
        if (!isVideo(path))
            return path;
        const sibling = path.replace(/\.[^./]+$/, ".poster.jpg");
        const cached = `${Theme.stateDir}/posters/${Qt.md5(path)}.jpg`;
        return posters[sibling] ? sibling : posters[cached] ? cached : "";
    }

    // Folders first: the file listing needs their directories.
    function refresh(): void {
        folderLister.running = true;
    }

    // The folder a wallpaper file is in, or "".
    function folderOf(path: string): string {
        const dir = path.slice(0, path.lastIndexOf("/"));
        return folders.find(f => f.dir && f.dir === dir)?.name ?? "";
    }

    function set(path: string): void {
        Quickshell.execDetached(["dot-theme", "set", path]);
    }

    // Display name: file name without extension, dashes as spaces.
    function label(path: string): string {
        return path.split("/").pop().replace(/\.[^.]+$/, "").replace(/[-_]+/g, " ");
    }

    // find(1) arguments matching any of the extensions: ( -iname *.a -o -iname *.b ).
    function nameArgs(exts: list<string>): list<string> {
        const args = [];
        for (const e of exts)
            args.push("-o", "-iname", `*.${e}`);
        return ["(", ...args.slice(1), ")"];
    }

    // Posters must be known before the desktop draws a video's first frame.
    Component.onCompleted: refresh()
    // dot-theme just cut a poster for a video that had none: pick it up.
    onCurrentChanged: if (listed && isVideo(current) && !poster(current))
        refresh()

    FileView {
        id: currentFile

        path: `${Theme.stateDir}/wallpaper`
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
    }

    Process {
        id: folderLister

        command: ["dot-theme", "folders"]
        stdout: StdioCollector {
            onStreamFinished: {
                root.folders = text.split("\n").filter(Boolean).map(l => {
                    const [name, dir] = l.split("\t");
                    return {
                        name,
                        dir
                    };
                });
                wallpaperLister.running = true;
            }
        }
    }

    // Which wallpapers each folder shows is dot-theme's to say: a set can be
    // picked one wallpaper at a time (the sets file), so a folder's files
    // aren't all offered.
    Process {
        id: wallpaperLister

        command: ["dot-theme", "wallpapers"]
        stdout: StdioCollector {
            onStreamFinished: {
                const found = {};
                for (const f of root.folders)
                    found[f.name] = [];
                for (const line of text.split("\n").filter(Boolean)) {
                    const tab = line.indexOf("\t");
                    found[line.slice(0, tab)]?.push(line.slice(tab + 1));
                }
                root.images = found;
                posterLister.running = true;
            }
        }
    }

    Process {
        id: posterLister

        // -L: the sets are a link into the Nix store.
        command: ["find", "-L", ...root.folders.map(f => f.dir), `${Theme.stateDir}/posters`, "-maxdepth", "1", "-type", "f", ...root.nameArgs(root.imageExtensions)]
        stdout: StdioCollector {
            onStreamFinished: {
                const posters = {};
                for (const path of text.split("\n").filter(Boolean)) {
                    if (/\.poster\.[^./]+$/i.test(path) || path.startsWith(`${Theme.stateDir}/posters/`))
                        posters[path] = true;
                }
                root.posters = posters;
                root.listed = true;
            }
        }
    }
}

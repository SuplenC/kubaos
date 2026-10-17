pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import qs.theme
import qs.settings

// Current wallpaper (from ~/.local/state/theme/wallpaper, written by dot-theme)
// and the wallpapers available in each folder: stills, and videos that loop
// over a poster frame (<name>.poster.jpg next to <name>.mp4; dot-theme makes
// one in ~/.local/state/theme/posters/ for a video that has none). ukiyo's
// live scenes are wallpapers too, `ukiyo:<id>`, listed first in
// Settings.sceneFolder; their still is the scene's poster (services/Ukiyo.qml).
Singleton {
    id: root

    readonly property string current: currentFile.text().trim() || (Theme.palette.wallpaper ?? "")
    // Shown on the desktop instead of `current` while the picker browses
    // (colors only change once a wallpaper is applied). "" = no preview.
    property string preview: ""
    readonly property string shown: preview || current
    // The picker opens on its settings tab next time (`qs ipc call wallpaper settings`).
    property bool openSettings: false
    // folder name -> sorted list of absolute paths (stills and videos), the
    // scenes (ukiyo:<id>) first in Settings.sceneFolder
    readonly property var images: {
        const all = Object.assign({}, files);
        const scenes = Ukiyo.scenes.map(s => `ukiyo:${s.id}`);
        if (scenes.length)
            all[Settings.sceneFolder] = [...scenes, ...(files[Settings.sceneFolder] ?? [])];
        return all;
    }
    // folder name -> sorted list of the files in it
    property var files: ({})
    // Poster files next to videos and in dot-theme's cache, as a set (path -> true).
    property var posters: ({})
    // The first listing is in: `poster()` knows what exists.
    property bool listed: false
    readonly property list<string> videoExtensions: ["mp4", "webm", "mkv", "mov"]
    readonly property list<string> imageExtensions: ["jpg", "jpeg", "png", "webp"]

    function isScene(path: string): bool {
        return path.startsWith("ukiyo:");
    }

    function isVideo(path: string): bool {
        return videoExtensions.includes(path.split(".").pop().toLowerCase());
    }

    // The still that stands in for a video: the picker thumbnail, what shows
    // before the first frame and whenever motion is off. "" when it has none
    // (or before the first listing). Must match poster_for in scripts/dot-theme.
    function poster(path: string): string {
        if (isScene(path))
            return Ukiyo.poster(path.slice(6));
        if (!isVideo(path))
            return path;
        const sibling = path.replace(/\.[^./]+$/, ".poster.jpg");
        const cached = `${Theme.stateDir}/posters/${Qt.md5(path)}.jpg`;
        return posters[sibling] ? sibling : posters[cached] ? cached : "";
    }

    function refresh(): void {
        lister.running = true;
        Ukiyo.refreshScenes();
    }

    function set(path: string): void {
        Quickshell.execDetached(["dot-theme", "set", path]);
    }

    // Display name: file name without extension, dashes as spaces.
    function label(path: string): string {
        if (isScene(path))
            return Ukiyo.name(path.slice(6));
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
        id: lister

        command: ["find", ...Settings.wallpaperFolders.map(f => `${Settings.wallpaperRoot}/${f}`), `${Theme.stateDir}/posters`, "-maxdepth", "1", "-type", "f", ...root.nameArgs([...root.imageExtensions, ...root.videoExtensions])]
        stdout: StdioCollector {
            onStreamFinished: {
                const found = {};
                for (const folder of Settings.wallpaperFolders)
                    found[folder] = [];
                const posters = {};
                for (const path of text.split("\n").filter(Boolean)) {
                    if (/\.poster\.[^./]+$/i.test(path) || path.startsWith(`${Theme.stateDir}/posters/`)) {
                        posters[path] = true;
                        continue;
                    }
                    const folder = path.slice(Settings.wallpaperRoot.length + 1).split("/")[0];
                    found[folder]?.push(path);
                }
                for (const folder in found)
                    found[folder].sort();
                root.posters = posters;
                root.files = found;
                root.listed = true;
            }
        }
    }
}

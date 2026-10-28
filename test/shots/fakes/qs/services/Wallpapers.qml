pragma Singleton
import QtQuick

// Stand-in for Wallpapers: the pinned wallpaper sets (`@WALLPAPERS@`, filled in when a tree is
// staged), the neon sign current.
QtObject {
    id: root

    property string current: "@WALLPAPERS@/shinjuku/kubaos-neon-sign.jpg"
    property string preview: ""
    readonly property string shown: preview || current
    property bool openSettings: false
    property bool listed: true
    readonly property list<string> videoExtensions: ["mp4", "webm", "mkv", "mov"]
    readonly property list<string> imageExtensions: ["jpg", "jpeg", "png", "webp"]
    property var folders: [
        {
            name: "shinjuku",
            dir: "@WALLPAPERS@/shinjuku"
        },
        {
            name: "cyberpunk",
            dir: "@WALLPAPERS@/cyberpunk"
        },
        {
            name: "japanese",
            dir: "@WALLPAPERS@/japanese"
        }
    ]
    property var images: ({
            shinjuku: ["@WALLPAPERS@/shinjuku/kabukicho-red-gate.jpg", "@WALLPAPERS@/shinjuku/kubaos-neon-sign.jpg", "@WALLPAPERS@/shinjuku/nishi-shinjuku-violet-rain.jpg", "@WALLPAPERS@/shinjuku/omoide-yokocho-lanterns.jpg", "@WALLPAPERS@/shinjuku/skyline-fuji-dusk.jpg"],
            cyberpunk: ["@WALLPAPERS@/cyberpunk/akihabara-after-rain.jpg", "@WALLPAPERS@/cyberpunk/shibuya-rainy-corner.jpg", "@WALLPAPERS@/cyberpunk/shinjuku-neon-canyon.jpg"],
            japanese: ["@WALLPAPERS@/japanese/shinjuku-purple-hour.jpg", "@WALLPAPERS@/japanese/tokyo-tower-blue-hour.jpg"]
        })
    property var posters: ({})

    function isVideo(path: string): bool {
        return root.videoExtensions.some(e => path.endsWith(`.${e}`));
    }

    function poster(path: string): string {
        return path;
    }

    function refresh(): void {
    }

    function folderOf(path: string): string {
        const dir = path.slice(0, path.lastIndexOf("/"));
        return root.folders.find(f => f.dir === dir)?.name ?? "";
    }

    function set(path: string): void {
        root.current = path;
    }

    function label(path: string): string {
        return path.split("/").pop().replace(/\.[^.]+$/, "").replace(/[-_]+/g, " ");
    }

    function nameArgs(exts: var): var {
        return [];
    }
}

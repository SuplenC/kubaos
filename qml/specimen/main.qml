import QtQuick
import KubaOS.Components as K

// Lays the QML specimen out headless and reports it, for the cross-toolkit test
// (`kubaos-specimen parity`) and for screenshots to set beside `kubaos-specimen`'s:
//
//   qml main.qml -- WIDTH MODE [PNG]
//
// WIDTH is the page's width in logical pixels, MODE `dark` or `light`; the scale is the
// display's (`QT_SCALE_FACTOR`). Once the layout has settled, the page's geometry frame is printed
// as one line starting `frame `, the screenshot written to PNG when given, and the runner quits.
//
// The "Shell only" page (`ShellOnly.qml`) is laid out under the components page and is in the
// screenshot, but not in the frame: its parts have no iced twin for parity to hold them to.
Window {
    id: window

    readonly property var arguments: Qt.application.arguments.slice(Qt.application.arguments.indexOf("--") + 1)
    readonly property real pageWidth: Number(window.arguments[0])
    readonly property string shot: window.arguments[2] ?? ""

    width: window.pageWidth
    height: specimen.height + shellOnly.height
    visible: true
    color: K.Tokens.color.surface

    // The page's boxes as the geometry check reads them: every item that says what it is, from
    // the page's top left, with the items that say nothing left out and their children kept. A
    // layout's child that says nothing is a plain box, since the layout's rules hold its edges,
    // and a plain box around one child with its bounds is that child, as iced reports them.
    function node(item: Item, into: var): void {
        if (!item.visible || item.overlay === true)
            return;
        const laidOut = item.parent && typeof item.parent.managed === "function" && typeof item.itemAt !== "function";
        const role = item.probe ? item.probe : laidOut ? {
            role: "box"
        } : null;
        if (!role) {
            for (const child of item.children)
                window.node(child, into);
            return;
        }
        const at = item.mapToItem(page, 0, 0);
        const bounds = {
            x: at.x,
            y: at.y,
            width: item.width,
            height: item.height
        };
        let children = [];
        for (const child of item.children)
            window.node(child, children);
        const same = other => ["x", "y", "width", "height"].every(edge => other.bounds[edge] === bounds[edge]);
        while (children.length === 1 && children[0].role === "box" && same(children[0]))
            children = children[0].children;
        if (role.role === "box" && children.length === 1 && same(children[0])) {
            into.push(children[0]);
            return;
        }
        into.push(Object.assign({}, role, {
            bounds: bounds,
            children: children
        }));
    }

    // The page's frame, or null when its tree doesn't come down to one root, which the geometry
    // check needs.
    function frame(): var {
        const roots = [];
        window.node(specimen, roots);
        if (roots.length !== 1) {
            console.error(`the page has ${roots.length} roots, not one`);
            return null;
        }
        return {
            page: "components",
            mode: K.Tokens.mode,
            scale: window.devicePixelRatio,
            root: roots[0]
        };
    }

    Component.onCompleted: K.Tokens.mode = window.arguments[1] === "light" ? "light" : "dark"

    // The page on the surface, as `kubaos-specimen` draws it.
    Rectangle {
        id: page

        width: window.width
        height: specimen.height + shellOnly.height
        color: K.Tokens.color.surface

        Components {
            id: specimen

            width: window.width
            shownScale: window.devicePixelRatio
        }

        ShellOnly {
            id: shellOnly

            y: specimen.height
            width: window.width
            height: shellOnly.implicitHeight
        }
    }

    // How many layouts the page has: every item that arranges children.
    function layouts(item: Item): int {
        let n = typeof item.arrange === "function" ? 1 : 0;
        for (const child of item.children)
            n += window.layouts(child);
        return n;
    }

    // One pass over the whole page, as a resize makes it: how many times a layout arranged its
    // children, printed as `passes N layouts M ms T` for `qml-specimen` to hold to a bound.
    function measurePass(): void {
        const narrower = window.width - K.Tokens.space.s4;
        K.LayoutPasses.count = 0;
        const start = Date.now();
        specimen.width = narrower;
        specimen.relayout();
        const passes = K.LayoutPasses.count;
        const took = Date.now() - start;
        specimen.width = window.width;
        specimen.relayout();
        console.log(`passes ${passes} layouts ${window.layouts(specimen)} ms ${took}`);
    }

    // Lays out until two reports in a row agree, measures a pass, then lays out until two reports
    // agree again and reports.
    Timer {
        property string last: ""
        property bool measured: false

        interval: 100
        repeat: true
        running: true
        onTriggered: {
            specimen.relayout();
            shellOnly.relayout();
            const frame = window.frame();
            if (frame === null) {
                Qt.exit(1);
                return;
            }
            // The "Shell only" page settles with the frame: its height is in what is compared.
            const report = `${JSON.stringify(frame)} ${shellOnly.height}`;
            if (report !== last) {
                last = report;
                return;
            }
            if (!measured) {
                measured = true;
                last = "";
                window.measurePass();
                return;
            }
            running = false;
            console.log(`frame ${JSON.stringify(frame)}`);
            if (window.shot === "") {
                Qt.quit();
                return;
            }
            // The window's whole content: hints float over the page on it, as iced's overlay
            // does.
            window.contentItem.grabToImage(result => {
                result.saveToFile(window.shot);
                Qt.quit();
            });
        }
    }
}

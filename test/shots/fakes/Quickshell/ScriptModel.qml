import QtQuick

// Stand-in for ScriptModel: a ListModel whose `modelData` role is each value. It rebuilds only
// when the values change, as the real one keeps the rows it still has; with `objectProp`, values
// with the same keys in the same order keep their rows and take the new values in place, as the
// real one does, so a delegate's state (a notification group's preview) outlives a new value.
ListModel {
    id: model

    property var values: []
    // The property a value is kept by as it changes.
    property string objectProp: ""
    property var shown: []

    function keyOf(v: var): var {
        return model.objectProp !== "" && v !== null && typeof v === "object" ? v[model.objectProp] : v;
    }

    function sync(): void {
        const now = model.values ?? [];
        if (now.length === model.shown.length && now.every((v, i) => v === model.shown[i]))
            return;
        const sameKeys = model.objectProp !== "" && now.length === model.shown.length && now.every((v, i) => model.keyOf(v) === model.keyOf(model.shown[i]));
        model.shown = now.slice();
        if (sameKeys) {
            now.forEach((v, i) => model.setProperty(i, "modelData", v));
            return;
        }
        model.clear();
        for (const v of now)
            model.append({
                modelData: v
            });
    }

    onValuesChanged: sync()
    Component.onCompleted: sync()
}

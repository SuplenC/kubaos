import QtQuick

// Stand-in for ScriptModel: a ListModel whose `modelData` role is each value. It rebuilds only
// when the values change, as the real one keeps the rows it still has.
ListModel {
    id: model

    property var values: []
    // The real one keeps a row by this property as its value changes; this one rebuilds.
    property string objectProp: ""
    property var shown: []

    function sync(): void {
        const now = model.values ?? [];
        if (now.length === model.shown.length && now.every((v, i) => v === model.shown[i]))
            return;
        model.shown = now.slice();
        model.clear();
        for (const v of now)
            model.append({
                modelData: v
            });
    }

    onValuesChanged: sync()
    Component.onCompleted: sync()
}

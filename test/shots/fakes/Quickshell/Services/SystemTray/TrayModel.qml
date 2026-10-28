import QtQuick

// The tray's items as a model whose `modelData` role is each item, with `values` beside it.
ListModel {
    id: model

    property list<QtObject> source
    readonly property var values: Array.from(model.source)

    Component.onCompleted: {
        for (const item of model.source)
            model.append({
                modelData: item
            });
    }
}

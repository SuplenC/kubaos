import QtQuick
import QtTest
import KubaOS.Components as K

// The keyed diff a live list applies to its model: what it removes, inserts and moves.
TestCase {
    id: suite

    name: "keyed"

    ListModel {
        id: model
    }

    // Applies `ops` to `keys`, as a ListModel would.
    function play(keys, ops) {
        const out = keys.slice();
        for (const op of ops) {
            if (op.op === "remove")
                out.splice(op.at, 1);
            else if (op.op === "insert")
                out.splice(op.at, 0, op.key);
            else
                out.splice(op.to, 0, out.splice(op.from, 1)[0]);
        }
        return out;
    }

    function kinds(ops) {
        return ops.map(op => op.op);
    }

    function fill(keys) {
        model.clear();
        for (const key of keys)
            model.append({
                key: key,
                title: key
            });
    }

    function keysOf() {
        const out = [];
        for (let i = 0; i < model.count; i++)
            out.push(model.get(i).key);
        return out;
    }

    function test_the_same_keys_need_no_ops() {
        compare(K.Keyed.diff(["a", "b", "c"], ["a", "b", "c"]), []);
    }

    function test_a_new_key_is_one_insert_where_it_goes() {
        const ops = K.Keyed.diff(["a", "c"], ["a", "b", "c"]);

        compare(kinds(ops), ["insert"]);
        compare(ops[0].at, 1);
        compare(play(["a", "c"], ops), ["a", "b", "c"]);
    }

    function test_a_gone_key_is_one_remove() {
        const ops = K.Keyed.diff(["a", "b", "c"], ["a", "c"]);

        compare(kinds(ops), ["remove"]);
        compare(ops[0].at, 1);
    }

    function test_one_row_moving_past_the_rest_is_one_move() {
        const ops = K.Keyed.diff(["a", "b", "c", "d"], ["b", "c", "d", "a"]);

        compare(kinds(ops), ["move"]);
        compare(play(["a", "b", "c", "d"], ops), ["b", "c", "d", "a"]);
    }

    function test_a_row_climbing_to_the_top_is_one_move() {
        const ops = K.Keyed.diff(["a", "b", "c", "d"], ["d", "a", "b", "c"]);

        compare(kinds(ops), ["move"]);
        compare(play(["a", "b", "c", "d"], ops), ["d", "a", "b", "c"]);
    }

    function test_removes_come_before_inserts_and_moves() {
        const from = ["a", "b", "c", "d", "e"];
        const to = ["f", "d", "a", "g", "c"];

        const ops = K.Keyed.diff(from, to);

        compare(play(from, ops), to);
        const firstOther = ops.findIndex(op => op.op !== "remove");
        verify(ops.slice(firstOther).every(op => op.op !== "remove"));
    }

    function test_every_shuffle_lands_on_its_target() {
        const keys = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let seed = 7;
        const random = () => {
            seed = (seed * 1103515245 + 12345) % 2147483648;
            return seed / 2147483648;
        };
        for (let round = 0; round < 200; round++) {
            const from = keys.filter(() => random() < 0.7);
            const to = keys.filter(() => random() < 0.7).sort(() => random() - 0.5);

            compare(play(from, K.Keyed.diff(from, to)), to, `${from} to ${to}`);
        }
    }

    function test_apply_brings_a_model_to_the_rows() {
        fill(["a", "b", "c"]);

        K.Keyed.apply(model, [
            {
                key: "c",
                title: "C"
            },
            {
                key: "a",
                title: "a"
            },
            {
                key: "d",
                title: "d"
            }
        ]);

        compare(keysOf(), ["c", "a", "d"]);
        compare(model.get(0).title, "C");
        compare(model.get(2).title, "d");
    }

    function test_apply_moves_a_row_rather_than_making_it_again() {
        fill(["a", "b", "c"]);
        let inserts = 0;
        let moves = 0;
        const inserted = () => inserts++;
        const moved = () => moves++;
        model.rowsInserted.connect(inserted);
        model.rowsMoved.connect(moved);

        K.Keyed.apply(model, [
            {
                key: "c",
                title: "c"
            },
            {
                key: "a",
                title: "a"
            },
            {
                key: "b",
                title: "b"
            }
        ]);

        model.rowsInserted.disconnect(inserted);
        model.rowsMoved.disconnect(moved);
        compare(inserts, 0);
        compare(moves, 1);
    }

    function test_apply_leaves_an_unchanged_row_alone() {
        fill(["a"]);
        let changes = 0;
        const count = () => changes++;
        model.dataChanged.connect(count);

        K.Keyed.apply(model, [
            {
                key: "a",
                title: "a"
            }
        ]);

        model.dataChanged.disconnect(count);
        compare(changes, 0);
    }
}

pragma Singleton

import QtQuick
import KubaOS.Components as K

// What `Button` and `IconButton` share: each size's measures, all from the scale, and each kind's
// colours, dimmed when disabled.
QtObject {
    // A size's height, label style, icon, gap between icon and label, and padding at the sides of
    // a filled, tonal or danger button and of a text one, which has no edge to keep the label from.
    function of(size: string): var {
        switch (size) {
        case "Sm":
            return {
                height: Tokens.size.control_sm,
                label: Tokens.type.button_small,
                icon: Tokens.type.button_small.icon,
                gap: Tokens.space.s4,
                padding: Tokens.space.s12,
                textPadding: Tokens.space.s12
            };
        case "Lg":
            return {
                height: Tokens.size.control_lg,
                label: Tokens.type.button_large,
                icon: Tokens.type.button_large.icon,
                gap: Tokens.space.s8,
                padding: Tokens.space.s32,
                textPadding: Tokens.space.s16
            };
        default:
            return {
                height: Tokens.size.control_md,
                label: Tokens.type.button,
                icon: Tokens.type.button.icon,
                gap: Tokens.space.s8,
                padding: Tokens.space.s24,
                textPadding: Tokens.space.s12
            };
        }
    }

    // A kind's fill (transparent for `Text`), its content colour, and the colour the label and icon
    // show in: the content's, or the dimmed one when disabled.
    function colours(kind: string, enabled: bool): var {
        const c = Tokens.color;
        const pair = kind === "Tonal" ? [c.secondary_container, c.on_secondary_container] : kind === "Text" ? ["transparent", c.primary] : kind === "Danger" ? [c.error, c.on_error] : [c.primary, c.on_primary];
        if (enabled)
            return {
                fill: pair[0],
                content: pair[1],
                shown: pair[1]
            };
        return {
            fill: kind === "Text" ? "transparent" : K.Look.alpha(c.on_surface, Tokens.opacity.disabled_container),
            content: pair[1],
            shown: K.Look.alpha(c.on_surface, Tokens.opacity.disabled_content)
        };
    }
}

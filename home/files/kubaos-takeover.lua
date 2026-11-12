-- New devices take over: while the `kubaos.takeover` setting is on, a headset
-- connected over Bluetooth or USB becomes the default output, and the default
-- input once it has a microphone, as if it were picked by hand. WirePlumber
-- then remembers it as the user's choice, so unplugging it goes back to the
-- one before. `config/wireplumber/wireplumber.conf.d/50-kubaos.conf` loads it;
-- `scripts/dot-audio takeover` sets the setting.
--
-- Only Bluetooth and USB: a screen's HDMI audio appears with the screen,
-- which is no wish to listen through it.
--
-- https://pipewire.pages.freedesktop.org/wireplumber/scripting/lua_api.html

local log = Log.open_topic ("s-kubaos-takeover")

local BUSES = { bluetooth = true, usb = true }
local KINDS = { ["Audio/Sink"] = "audio.sink", ["Audio/Source"] = "audio.source" }

-- WirePlumber adds every device already there in its first moments after it
-- starts: those were present at login, not connected.
local SETTLE_MSEC = 5000
local settled = false
Core.timeout_add (SETTLE_MSEC, function ()
  settled = true
  return false
end)

-- The devices connected since, by bound id, each with the kinds of node
-- that have yet to take over. A device takes over once per connection: a
-- Bluetooth headset's nodes come and go with its profile, and switching it
-- to Calls must not take back an output the user has since moved away.
local fresh = {}

SimpleEventHook {
  name = "kubaos/takeover/device-added",
  interests = {
    EventInterest {
      Constraint { "event.type", "=", "device-added" },
      Constraint { "media.class", "=", "Audio/Device" },
    },
  },
  execute = function (event)
    local device = event:get_subject ()
    if not settled or not Settings.get_boolean ("kubaos.takeover") then
      return
    end
    if not BUSES [device.properties ["device.bus"] or ""] then
      return
    end
    fresh [device ["bound-id"]] = { ["audio.sink"] = true, ["audio.source"] = true }
  end
}:register ()

SimpleEventHook {
  name = "kubaos/takeover/device-removed",
  interests = {
    EventInterest {
      Constraint { "event.type", "=", "device-removed" },
      Constraint { "media.class", "=", "Audio/Device" },
    },
  },
  execute = function (event)
    fresh [event:get_subject () ["bound-id"]] = nil
  end
}:register ()

SimpleEventHook {
  name = "kubaos/takeover/node-added",
  interests = {
    EventInterest {
      Constraint { "event.type", "=", "node-added" },
      Constraint { "device.id", "+" },
    },
  },
  execute = function (event)
    local props = event:get_subject ().properties
    local kind = KINDS [props ["media.class"] or ""]
    local pending = fresh [tonumber (props ["device.id"])]
    if kind == nil or pending == nil or not pending [kind] then
      return
    end
    pending [kind] = nil

    local metadata = event:get_source ():call ("get-object-manager", "metadata"):lookup {
      Constraint { "metadata.name", "=", "default" },
    }
    if metadata == nil then
      return
    end
    log:info ("taking over " .. kind .. " with " .. props ["node.name"])
    -- The key and value `wpctl set-default` writes.
    metadata:set (0, "default.configured." .. kind, "Spa:String:JSON",
        Json.Object { ["name"] = props ["node.name"] }:to_string ())
  end
}:register ()

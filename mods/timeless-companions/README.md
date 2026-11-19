# Timeless Companions (vana-client mod)

TimelessXI's companions for vana-client, live. The two windows replace the Ashita addons companionview and
companionmanager and talk to the server the same way, so the server needs no change. The world touches are things
only vana-client can do.

- **In the world:**
  - companions' name plates in their role's colour (blue tank, green healer, violet support, red damage);
  - a ring in that colour at their feet;
  - what they say in party chat, shown in a speech bubble over their head.
- **Job icons:** the modern UI's own job pictograms, the ones its party window shows beside each member. They appear:
  - on Companion View's cards and header;
  - on the manager's party and kept rows, its call tiles, presets and sub jobs.

  Window canvases ask the client to draw icons of kind `"job"`. Only the Timeless fork has that kind (`mod.job_icons`; build it with `build-dye.bat`).
  Elsewhere a lettered badge stands in.
- **Live data:** HP, MP, TP, targets, spells, weapon skills, abilities and status effects come from the client itself
  (`lib/live.lua`), between server feeds. The windows show them too, and Companion View has a Target button.
- **Keys:** settings to bind Companion View and Companion Manager to keys. Each world touch can be switched off
  in the settings.

- **/cv: Companion View.** Your companions as cards. For the one you pick, it shows:
  - name, role and bond;
  - job and sub job, race;
  - HP, MP and TP;
  - the 4 × 4 equipment grid with item icons (point at one for its name, description, damage and delay, level and
    jobs);
  - combat stats, and attributes with gear bonuses in green or red.

  Thieves get a Treasure Hunter / Damage gear switch. Warriors, Paladins and Dark Knights get a Tank / Damage role
  switch.
- **/cmgr: Companion Manager.** (`/cm` is the game's own command.) Six tabs:
  - **Party:** Fill party, Two-hour: all and Dismiss all. Per companion: Keep, 2hr, Why and Dismiss. Downed
    companions show their time left, with Raise.
  - **Call:** every job by role, ready or on cooldown, with the Everglass price where one applies.
  - **Kept:** bond rank and progress, where each companion is, and Call, Errand and Forget. Collect appears when
    someone is back from an errand.
  - **Presets:** load, edit, delete, save your party, or build a preset from jobs.
  - **Tactics:** each job's rules.
  - **Settings:** engage, Treasure Hunter, nuke mode, party chat, roles, sub jobs, and the in-chat reports.

The server's answers to buttons show at the top of the manager for 30 seconds. Dismiss, Forget and Delete ask first.

Commands:
- `/cv`, `/cmgr` show or hide each window.
- `/cv update`, `/cmgr update` ask for fresh data now.
- `/cmgr party|call|kept|presets|tactics|settings` opens the manager on that tab.

## Install

1. Copy the `timeless-companions` folder into vana-client's mods folder: `%APPDATA%\vana-client\mods` on Windows.
2. In game, open Config → Settings → Mods and switch **Timeless Companions** on. Then type `/lua load timeless-companions`,
   or log in again.
3. On its page under Config → Extras, allow **packets.write**, and **actions** (for Companion View's Target button). The windows send their buttons to the server as the
   companions module's action packet, never as chat.
4. Optionally, on the same page, set **Auto Refresh** to have open windows update by themselves every so many seconds.
   It's off by default.

## How it works

- **Buttons:** a button sends its `!companion` command in a 0x0B5 packet that starts with `\x7F"CM|"` (`lib/action.lua`).
  The server's `onManagerAction` (`companions.cpp`) runs it and consumes the packet. This is the same as the Ashita
  addons, and typed `!companion` commands still work.
- **Feeds:** the server answers with feed lines, `CV1|...` (`!companion data`) and `CM1|...` (`!companion mdata`). A chat
  filter reads them and hides them (`lib/link.lua`; the parsers `lib/feed_cv.lua` and `lib/feed_cm.lua` are the addons'
  own).
- **When it asks:** nothing at login. A window asks when it opens, on Refresh, after its buttons, again 8 s after
  actions that swap companions in and out, and every Auto Refresh seconds if that's set. Companion View also asks when
  the manager's feed shows a different party.
- **Drawing:** the windows are drawn on canvases (`lib/canvas.lua`): cards, pill buttons, bars, icons, and tooltips
  with hover highlights.

## Tests

`lua tests/run.lua [folder]` loads the mod in a stand-in for the client's API (`tests/harness.lua`). It runs 149
checks:
- feeds the server's lines through it;
- clicks every kind of button and checks the packet it sends;
- checks every drawn command stays inside its canvas.

With a folder, it also writes each window's draw lists as JSON, for previews.

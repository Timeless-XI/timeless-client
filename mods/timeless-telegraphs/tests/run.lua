-- Timeless Telegraphs' tests: lua tests/run.lua (from the mod folder).

local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end
local function near(a, b, eps) return math.abs(a - b) <= (eps or 1e-6) end

H.mod_id = "timeless-telegraphs"
-- A crab 5 yalms east of us, facing west (towards us); we stand at the origin facing east
H.player.pos = { x = 0, y = -10, z = 0 }
H.player.heading = 0
H.entities = {
  [17000100] = { id = 17000100, index = 100, name = "Crab", kind = "mob", pos = { x = 5, y = -10, z = 0 }, heading = math.pi, hpp = 80 },
}

dofile("init.lua")
local tg = require("lib.state")
local protocol = require("lib.protocol")
local shapes = require("lib.shapes")

ok(H.commands.telegraphs and H.commands.timelesstelegraphs, "commands")
ok(H.provided["timeless-telegraphs"].timeless_menu().title == "Telegraphs", "a TimelessMenu card")

-- Asking: "tg on" soon after load, then every 20 s
H.run(1.6)
local s = H.take_sent()
ok(#s == 1 and s[1] == "tg on", "tg on after load: " .. table.concat(s, ","))
H.run(20)
ok(#H.take_sent() == 1, "heartbeat every 20 s")

-- The parser
local t = protocol.parse(protocol.extract("\30\1TG1|100|s=f,r=12.0,a=45,t=1,ms=2500,n=Fire Breath\127"), 10)
ok(t and t.mob == 100 and t.shape == "f" and t.radius == 12 and t.angle == 45 and t.target == 1 and t.name == "Fire Breath", "parsed")
ok(near(t.ends, 12.5), "ends when the move goes off")
ok(protocol.parse("TG1|100|s=x,r=5,ms=1000", 0) == nil, "unknown shape refused")
ok(protocol.parse("TG1|100|s=c,r=0,ms=1000", 0) == nil, "no radius refused")
ok(protocol.parse("TG1|100|s=c,r=5,ms=0", 0) == nil, "instant refused")
ok(protocol.parse("TG1|100|s=c,r=500,ms=99999", 0).radius == 60, "huge radius capped")
ok(protocol.parse("TG1|100|s=c,r=5,ms=99999", 0).ends == 15, "huge ready time capped")

-- Geometry: headings, the cone's corners, inside tests
local dx, dz = shapes.dir(0)
ok(near(dx, 1) and near(dz, 0), "heading 0 points east")
dx, dz = shapes.dir(math.pi / 2)
ok(near(dx, 0) and near(dz, -1), "heading π/2 points south (-z)")
local a, b, c = shapes.cone_points({ x = 0, z = 0 }, 0, 10, 90)
ok(near(b.x, 10 * math.cos(math.pi / 4)) and near(c.x, b.x) and near(b.z, -c.z), "a 90° cone's ends are symmetric")
local cone = { shape = "f", radius = 10, angle = 45 }
local at = { centre = { x = 0, y = 0, z = 0 }, heading = 0 }
ok(shapes.inside(cone, at, { x = 5, z = 0 }), "straight ahead is inside the cone")
ok(not shapes.inside(cone, at, { x = -1, z = 0 }), "behind is outside")
ok(not shapes.inside(cone, at, { x = 11, z = 0 }), "past its reach is outside")
ok(not shapes.inside(cone, at, { x = 5, z = 3 }), "off to the side (beyond 22.5°) is outside")
ok(shapes.inside(cone, at, { x = 5, z = 1.5 }), "within 22.5° is inside")
local circle = { shape = "c", radius = 6 }
ok(shapes.inside(circle, at, { x = 4, z = 4 }) and not shapes.inside(circle, at, { x = 5, z = 4 }), "circle inside / outside")

-- A cone from the crab towards us: drawn, hidden from chat, and we are inside it
ok(H.line("TG1|100|s=f,r=12.0,a=45,t=1,ms=2500,n=Fire Breath") == false, "hidden from chat")
ok(H.line("A real chat line") == "A real chat line", "chat untouched")
ok(tg.active[100] ~= nil, "the telegraph is active")
H.fire("frame")
local img = H.draws.tg100f
ok(img and img.kind == "image" and img.path == "img/cone.png" and img.facing == "ground", "the cone's picture lies on the ground")
ok(near(img.yaw, math.pi) and near(img.at.x, 5 - 6, 1e-6) and near(img.h, 12) and near(img.w, 24 * math.tan(math.rad(22.5)), 1e-6),
  "pointing west from the crab, centred half way along, 12 long")
ok(img.at.y < -10 and img.at.y > -10.2, "a hair above the ground (y down)")
ok(H.draws.tg100l1 and H.draws.tg100l2 and H.draws.tg100l3 and H.draws.tg100w, "its edges and the sweeping front are drawn")
ok(H.draws.tg100n and H.draws.tg100n.text == "Fire Breath", "the move's name over it")
ok(H.hud.tg_alert and H.hud.tg_alert.text:find("Move out!", 1, true), "we are inside: the screen says move")

-- The front sweeps out over time
local function front_x() return (H.draws.tg100w.from.x + H.draws.tg100w.to.x) / 2 end
local x0 = front_x()
H.clock = H.clock + 1.25
H.fire("frame")
ok(front_x() < x0 - 5, "the front moved half way out (" .. x0 .. " -> " .. front_x() .. ")")

-- The picture isn't redrawn while the crab stands still; it is when it turns
local calls = 0
local real_image = draw.image
draw.image = function(id, spec) calls = calls + 1 real_image(id, spec) end
H.fire("frame")
ok(calls == 0, "a still cone keeps its picture")
H.entities[17000100].heading = math.pi + 0.3
H.fire("frame")
ok(calls == 1 and near(H.draws.tg100f.yaw, math.pi + 0.3), "the crab turned: the cone turns with it")
draw.image = real_image

-- We step out: the warning goes
H.player.pos = { x = 0, y = -10, z = 20 }
H.fire("frame")
ok(H.hud.tg_alert == nil, "out of it: no warning")

-- It goes off: a flash, then it's gone
H.clock = H.clock + 1.3
H.fire("frame")
ok(H.draws.tg100f ~= nil, "flashing as it goes off")
H.clock = H.clock + 0.4
H.fire("frame")
ok(H.draws.tg100f == nil and H.draws.tg100l1 == nil and H.draws.tg100n == nil and tg.active[100] == nil, "then gone, every piece")

-- A circle around the target (us): follows us
H.player.pos = { x = 0, y = -10, z = 0 }
H.line("TG1|100|s=t,r=6.0,a=45,t=1,ms=2000,n=Meteor")
H.fire("frame")
ok(H.draws.tg100e and H.draws.tg100e.kind == "ring" and near(H.draws.tg100e.at.x, 0) and H.draws.tg100e.radius == 6, "a ring around us")
ok(H.draws.tg100f.filled and H.draws.tg100p.filled and H.draws.tg100p.radius < 1, "a faint disc, and the fill just starting")
H.player.pos = { x = 3, y = -10, z = 0 }
H.fire("frame")
ok(near(H.draws.tg100e.at.x, 3), "it follows us")
ok(H.hud.tg_alert ~= nil, "we're inside our own circle")

-- A circle around the monster; a cone behind it
H.line("TG1|100|s=c,r=8.0,a=45,t=1,ms=2000,n=Stomp")
H.fire("frame")
ok(near(H.draws.tg100e.at.x, 5) and H.draws.tg100e.radius == 8, "a new move replaces the old marker: a ring around the crab")
H.line("TG1|100|s=b,r=9.0,a=45,t=1,ms=2000,n=Tail Swing")
H.fire("frame")
ok(near(H.draws.tg100f.yaw, 0.3, 1e-6), "a cone behind the crab points the other way")

-- The crab dies mid-ready: the marker goes
H.entities[17000100].hpp = 0
H.fire("frame")
ok(tg.active[100] == nil and H.draws.tg100f == nil, "a dead monster's marker goes")
H.entities[17000100].hpp = 80

-- Several monsters at once
H.entities[17000101] = { id = 17000101, index = 101, name = "Crab", kind = "mob", pos = { x = -5, y = -10, z = 0 }, heading = 0, hpp = 90 }
H.line("TG1|100|s=c,r=5.0,a=45,t=1,ms=2000,n=A")
H.line("TG1|101|s=f,r=7.0,a=45,t=1,ms=2000,n=B")
H.fire("frame")
ok(H.draws.tg100e and H.draws.tg101f, "two monsters, two markers")

-- Off: clears and tells the server; lines are dropped from chat but not drawn
H.take_sent()
H.command("/telegraphs")
H.run(0.1)
ok(next(tg.active) == nil and H.draws.tg100e == nil and H.draws.tg101f == nil, "off clears every marker")
ok(H.take_sent()[1] == "tg off", "off tells the server")
ok(H.line("TG1|100|s=c,r=5.0,a=45,t=1,ms=2000,n=A") == false and next(tg.active) == nil, "off: lines hidden, nothing drawn")
H.run(25)
ok(#H.take_sent() == 0, "no heartbeat while off")
H.command("/telegraphs")
H.run(0.1)
ok(H.take_sent()[1] == "tg on", "back on")

-- Options
H.command("/telegraphs alert")
H.line("TG1|100|s=c,r=8.0,a=45,t=1,ms=2000,n=Stomp")
H.player.pos = { x = 4, y = -10, z = 0 }
H.fire("frame")
ok(H.hud.tg_alert == nil, "alert off: no warning even inside")
H.command("/telegraphs alert")
H.command("/telegraphs labels")
H.line("TG1|100|s=c,r=8.0,a=45,t=1,ms=2000,n=Stomp")
H.fire("frame")
ok(H.draws.tg100n == nil, "labels off")
H.command("/telegraphs labels")
H.command("/telegraphs opacity 5")
ok(tg and storage.character.telegraphs.opacity == 1, "opacity clamped")

-- The test markers
H.command("/telegraphs test")
H.fire("frame")
ok(H.draws["tg-1e"] and H.draws["tg-2f"], "/telegraphs test draws a sample circle and cone")

-- Zoning and unloading clear everything
H.fire("zone_change")
ok(next(tg.active) == nil and H.draws["tg-1e"] == nil, "zoning clears")
H.take_sent()
H.fire("unload")
ok(H.take_sent()[1] == "tg off", "unload: tg off")

io.stdout:write(("ok: %d checks\n"):format(n))

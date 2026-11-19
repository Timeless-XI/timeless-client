-- Job icons: the modern UI's own job pictograms, the ones its party window shows beside each member.
--
-- They are drawn by the client itself as icons of kind "job" where it has them (the Timeless fork: mod.job_icons); elsewhere a plain badge
-- with the job's letters stands in.

local S = require("lib.style")

local J = {}

-- Whether the client draws icons of kind "job" (the Timeless fork). Unknown kinds draw an item's icon on a stock
-- client, so it must not be asked for one there.
J.native = type(mod) == "table" and mod.job_icons == true

-- Draws job `job`'s icon with its top left at (x, y), `size` across, on a canvas builder `c`.
-- o.role: the role whose colour the stand-in badge takes; o.dim: faded (a job that can't be called).
function J.draw(c, x, y, size, job, o)
  o = o or {}
  local k = o.dim and 0.45 or 1
  if J.native and job and job >= 1 and job <= 22 then
    c:icon(x, y, size, "job", job, { 255, 255, 255, math.floor(255 * k) })
    return
  end
  local base = (o.role or S.role_guess(job)).colour
  local r = size / 2
  c:circle(x + r, y + r, r, { math.floor(base[1] * 0.35), math.floor(base[2] * 0.35), math.floor(base[3] * 0.35), math.floor(235 * k) })
  c:circle(x + r, y + r, r - 0.5, S.fade(base, k), math.max(1, size / 16))
  c:text(x + r, y + r - 7 * size / 30, S.job_short(job), { size = size / 34, bold = true, align = "center",
    colour = S.fade({ 255, 248, 232, 255 }, k) })
end

return J

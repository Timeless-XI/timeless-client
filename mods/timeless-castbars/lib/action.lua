-- Asks the server without chat: a 0x0B5 packet whose text starts with \x7F"CM|", the same packet Timeless Companions
-- and the Ashita addons send. The server's companions module takes it and hands "cb ..." to the combat_feed module, so
-- nothing shows in chat.

local A = {}

A.PACKET = 0x0B5
A.MARKER = "\127CM|"
A.MAX = 120

-- The packet's body (after the 4-byte header): chat kind 0, a spare byte, the marker and text, a NUL.
function A.body(text)
  local clean = (text or ""):gsub("[^\32-\126]", "")
  return "\0\0" .. A.MARKER .. clean:sub(1, A.MAX) .. "\0"
end

function A.send(text)
  packets.inject_out(A.PACKET, A.body(text))
end

return A

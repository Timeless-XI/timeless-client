-- Asks the server without chat: a 0x0B5 packet whose text starts with \x7F"TA|". The server's timeless_artisan
-- module takes it (it never reaches chat) and answers with system lines starting "TA1|", which init.lua reads and
-- keeps out of the chat log. The same requests the old timelessartisan Ashita add-on sent.

local A = {}

A.PACKET = 0x0B5
A.MARKER = "\127TA|"
A.MAX = 116 -- the server reads 124 letters after the marker

-- The packet's body (after the 4-byte header): chat kind 0, a spare byte, the marker and text, a NUL.
function A.body(text)
  local clean = (text or ""):gsub("[^\32-\126]", "")
  return "\0\0" .. A.MARKER .. clean:sub(1, A.MAX) .. "\0"
end

function A.send(text)
  packets.inject_out(A.PACKET, A.body(text))
end

return A

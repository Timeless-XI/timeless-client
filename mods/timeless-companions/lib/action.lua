-- Sends a window's action to the server without chat: a 0x0B5 packet whose text starts with \x7F"CM|". The companions
-- module (companions.cpp, onManagerAction) takes it, runs it and consumes the packet, so it never shows as chat.
-- The same packet the Ashita addons (companionview, companionmanager) send.
--
-- The action text is the words of a !companion command ('keep Brokk', 'th on'), with:
--   fill     fill the party (plain !companion)
--   data     companionview's feed (CV1);  mdata  the manager's feed (CM1)
--   >words   the server answers in chat;  -words  no answer;  words  answers in the manager window, then a feed

local A = {}

A.PACKET = 0x0B5
A.MARKER = "\127CM|"
A.MAX = 120 -- letters of action text the server reads

-- '!companion keep Brokk' -> 'keep Brokk', '!companion' -> 'fill'. prefix: '', '>' or '-'.
function A.from_command(cmd, prefix)
  local text = (cmd or ""):gsub("^%s*!companion", ""):gsub("^%s+", ""):gsub("%s+$", "")
  if text == "" then text = "fill" end
  return (prefix or "") .. text
end

-- The packet's body (after the 4-byte header): chat kind 0, a spare byte, the marker and text, a NUL.
function A.body(text)
  local clean = (text or ""):gsub("[^\32-\126]", "")
  return "\0\0" .. A.MARKER .. clean:sub(1, A.MAX) .. "\0"
end

function A.send(text)
  packets.inject_out(A.PACKET, A.body(text))
end

return A

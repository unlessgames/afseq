local pattern = require "pattern"

---Wrap a lua integer index value into the given length
---@param index integer
---@param length integer
---@return integer
math.iwrap = function(index, length)
  return (index - 1) % length + 1
end

math.randomseed(0x12345)

return emitter {
  unit = "1/8",
  pattern = function(context)
    if math.iwrap(context.step, 8) == 1 then
      return { 0.8, 0.2, 0.9, 0.2 }
    else
      if math.random() > 0.9 then
        return { 0.8, 0.9 }
      else 
        return { 1 }
      end
    end
  end,
  emit = function(context)
    local note = "c6"
    if math.iwrap(context.step, context.step_count) == 3 then
      note = "c5 v0.3"
    end
    return note
  end
}
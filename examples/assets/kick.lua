return rhythm {
  unit = "1/16",
  pattern = function()
    local pulses = table.create({ 0, 6, 10 })
    ---@param context PatternContext
    return function(context)
      return pulses:find((context.pulse_step - 1) % 16) ~= nil
    end
  end,
  emit = { 60, 60, note { 60, { key = 96, volume = 0.135 } } },
}

return Emitter {
  unit = "sixteenth",
  offset = 16 * 8,
  pattern = euclidian(14, 16, 6),
  emit = function()
    local note_step = 0;
    return function()
      note_step = note_step + 1
      local volume = 1.0 - (note_step % 4) / 4.0
      local note = "C 5"
      if note_step % 3 == 0 then
        note = "C 4"
        volume = volume * 0.6
      end
      return { key = note, volume = volume }
    end
  end
}
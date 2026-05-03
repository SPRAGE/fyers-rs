Queue processing interval is a documented market-data socket configuration knob.
The supported range is 1ms to 2000ms; the Rust model rejects values outside
that range.

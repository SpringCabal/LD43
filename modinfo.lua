local modinfo = {
	name = "To the Last Drop WASM",
	shortname = "TTLDW",
	game = "To the Last Drop WASM",
	shortgame = "TTLDW",
	version = "$VERSION",
	description = "LD43 game",
	modtype = "1",
	-- Single-player only: without this the engine binds a UDP socket and warns
	-- that nobody else can reach it. Nothing here is ever hosted or joined.
	onlyLocal = true,
	depend = {
		"cursors.sdz",
	}
}

return modinfo

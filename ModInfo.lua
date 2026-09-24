local modinfo = {
	name = "To the last Drop",
	shortname = "LD43",
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

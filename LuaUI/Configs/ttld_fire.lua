return {
	-- Both playable ally teams have global LOS. Let the common GPU path
	-- clip billboards; avoid per-effect CPU visibility queries and masks.
	alwaysVisible = true,
	layer        = 0,
	speed        = 0.65,
	life         = 30,
	lifeSpread   = 10,
	delaySpread  = 20,
	colormap     = {
		{0, 0, 0, 0.},
		{0.4, 0.4, 0.4, 0.01},
		{0.35, 0.15, 0.15, 0.20},
		{0, 0, 0, 0}
	},
	rotSpeed     = 1,
	rotSpeedSpread = -2,
	rotSpread    = 360,
	sizeSpread   = 1,
	sizeGrowth   = 0.9,
	emitVector   = {0, 1, 0},
	emitRotSpread = 60,
	texture      = 'bitmaps/GPL/flame.png',
	count        = 5,

	-- fields that differ per instance (here for reference and static tables)
	force = {0, 1, 0},
	pos = {0, 0, 0},
	partpos = "",
	size = 1,
}


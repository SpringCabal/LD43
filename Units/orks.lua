
local OrkSmall = Humanoid:New {
	-- General
	name                = "OrkSmall",
	movementClass       = "Defender",
	objectName 			= "OrkSmall.dae",
	script              = "ork.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	weapons = {
		{
			name = "Axe",
		}
	},
}

local OrkBig = Humanoid:New {
	-- General
	name                = "OrkBig",
	movementClass       = "Defender",
	objectName 			= "OrkBig.dae",
	script              = "ork.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	weapons = {
		{
			name = "Axe",
		}
	},
}

local OrkBoss = Humanoid:New {
	-- General
	name                = "OrkBoss",
	movementClass       = "Defender",
	objectName 			= "OrkBoss.dae",
	script              = "ork.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	weapons = {
		{
			name = "Spear",
		}
	},
}


return {
	OrkSmall    = OrkSmall,
	OrkBig		= OrkBig,
	OrkBoss		= OrkBoss
}

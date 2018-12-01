local Peasant = Humanoid:New {
	-- General
	name                = "Peasant",
	movementClass       = "Defender",
	objectName 			= "HumanWithStick.dae",
	script              = "defender.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	weapons = {
		{
			name = "Spear",
		}
	},
}

local Crossbowman = Humanoid:New {
	-- General
	name                = "Crossbowman",
	movementClass       = "Defender",
	objectName 			= "HumanCrossbow.dae",
	script              = "defender.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	weapons = {
		{
			name = "Spear",
		}
	},
}

local Swordsman = Humanoid:New {
	-- General
	name                = "Swordsman",
	movementClass       = "Defender",
	objectName 			= "Swordsman.dae",
	script              = "defender.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	weapons = {
		{
			name = "Spear",
		}
	},
}

return {
	Swordsman   = Swordsman,
	Crossbowman = Crossbowman,
	Peasant     = Peasant
}

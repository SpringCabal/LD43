local Peasant = Humanoid:New {
	-- General
	name                = "Peasant",
	movementClass       = "Defender",
	objectName 			= "HumanWithStick.dae",
	script              = "defender.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	collisionVolumeScales    = '37 43 37',

	weapons = {
		{
			name = "DefenderSpear",
		}
	},
}

local Crossbowman = Humanoid:New {

  name                = [[Minotaur]],
  description         = [[Assault Tank]],
  acceleration        = 1.0237,
  brakeRate           = 0.04786,
  buildCostMetal      = 850,
  builder             = false,
  canGuard            = true,
  canMove             = true,
  canPatrol           = true,
  category            = [[LAND]],
  collisionVolumeOffsets = [[0 0 0]],
  collisionVolumeScales  = [[50 50 50]],
  collisionVolumeType    = [[ellipsoid]],  
  corpse              = [[DEAD]],

  customParams        = {
	aimposoffset   = [[0 0 0]],
	midposoffset   = [[0 0 0]],
	modelradius    = [[25]],
  },

  footprintX          = 4,
  footprintZ          = 4,
  idleAutoHeal        = 5,
  idleTime            = 1800,
  leaveTracks         = true,
  maxDamage           = 6800,
  maxSlope            = 18,
  maxVelocity         = 12.45,
  maxWaterDepth       = 22,
  minCloakDistance    = 75,
  noAutoFire          = false,
  sightDistance       = 506,
  trackOffset         = 8,
  trackStrength       = 8,
  trackStretch        = 1,
  trackWidth          = 42,
  turninplace         = 0,
  turnRate            = 364,
  workerTime          = 0,


	-- General
	name                = "Crossbowman",
	movementClass       = "Defender",
	objectName 			= "HumanCrossbow.dae",
	script              = "defender.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	collisionVolumeScales    = '37 64 37',

	weapons = {
		{
			name = "CrossBow",
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

	collisionVolumeScales    = '37 64 37',
	maxVelocity         = 3,
	footprintX 			= 3,
	footprintZ 			= 3,

	weapons = {
		{
			name = "DefenderSword",
		}
	},
}

return {
	Swordsman   = Swordsman,
	Crossbowman = Crossbowman,
	Peasant     = Peasant
}

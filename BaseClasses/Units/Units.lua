local Humanoid = Unit:New {
	-- FIXME: The docs for buildPic are a lie.
	-- Explicitly specifying an empty string gives different results.
	-- Nvm, this seems to be a side effect of using the OOP class system
	-- That said, it shouldn't have any defaults.
	buildPic 			= "",
	-- Sensors
	sightDistance       = 800,
	maxSlope              = 100,

	-- Commands
	canMove             = true,
	canAttack           = true,
	-- All these canX properties shouldn't be on by default:
	-- 	should they even be part of the engine? Lua it!
	canPatrol           = true,
	canGuard            = true,
	canRepeat           = true,
	footprintX          = 2,
	footprintZ          = 2,
	fireState           = 2,       -- Should auto-attack by default.
	mass                = 50,
	idleautoheal        = 0,
	autoheal            = 0,

	customParams = {
		hscale = 0.5,
		vscale = 0.5,
	},
	
	-- Movement & Placement
	upright             = true,
	minCollisionSpeed   = 1000000,
	pushResistant       = false,
	maxVelocity         = 8,
	acceleration        = 1.8,
	brakeRate           = 2,
	turnRate            = 1600 / 0.16,
	
	-- degrees per seconds = 0.16 * turnRate
	-- what a bizarre calculation, is this turnRate / (2 * pi)?
	-- this is probably just turnRate in radians and should be named such


	-- turnInPlace = true, (default)
	-- So it seems this default value is modeling a person rather than a car
	-- Why is the 'upright' also not reflecting that?
	-- Our default values should model *something*, otherwise they're pointless

	-- blocking & Spring.SetUnitBlocking
	-- we cannot modify all blocking properties here, and we need to rely on Lua

	-- Categories (probably don't need by default?)
	-- OH IT WAS NEEDED.
	category            = "INFANTRY",

	-- Collision Volumes
	-- This should be a good default for games that don't care about performance
	-- Let's hope it works...
	usePieceCollisionVolumes = false,
	usePieceSelectionVolumes = false,
	collisionVolumeType      = 'CylY',
	collisionVolumeScales    = '37 40 37',
}

local Raw = Unit:New {
}

return {
	Humanoid = Humanoid,
	Raw = Raw,
}
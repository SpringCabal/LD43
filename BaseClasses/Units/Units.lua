local Humanoid = Unit:New {
	-- FIXME: The docs for buildPic are a lie.
	-- Explicitly specifying an empty string gives different results.
	-- Nvm, this seems to be a side effect of using the OOP class system
	-- That said, it shouldn't have any defaults.
	buildPic 			= "",
	-- Sensors
	sightDistance       = 800,

	-- Commands
	canMove             = true,
	-- All these canX properties shouldn't be on by default:
	-- 	should they even be part of the engine? Lua it!
	canPatrol           = false,
	canGuard            = false,
	canRepeat           = false,
	fireState           = 2,       -- Should auto-attack by default.

	customParams = {
		hscale = 0.5,
		vscale = 0.5,
	},
	
	-- Movement & Placement
	-- Wiki: (this section should be split into building and non-building parts)
	-- Maybe even needs to be split differently for ships/air/ground
	footprintX          = 2, -- 1 seems a bad default (too small!)
	footprintZ          = 2,
	upright             = true,
	minCollisionSpeed   = 1000000,
	pushResistant       = true,
	maxVelocity         = 10,
	-- maxVelocity's default value of "0" is odd, unless the default is a "building"?
	-- then again, acceleration is not 0 by default..
	brakeRate           = 0.4,
	turnRate            = 360 / 0.16,
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
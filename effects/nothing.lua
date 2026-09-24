-- Explicitly empty explosion generator.  Referenced by the NOWEAPON placeholder
-- in GameData/weapondefs_pre.lua, which must never draw anything.
return {
	["nothing"] = {
		usedefaultexplosions = false,
	},
}

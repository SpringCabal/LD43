if not gadgetHandler:IsSyncedCode() then
	return
end

function gadget:GetInfo()
   return {
	  name      = "Blood Magic",
	  author    = "gajop",
	  date      = "December 2018",
	  layer     = 0,
	  enabled   = true
   }
end

local BLOOD_MAGIC_RANGE = 500
local MAX_DRAIN_PER_UNIT = 100

local function GetNearbyUnits(unitID)
	local x, _, z = Spring.GetUnitPosition(unitID)
	local units = Spring.GetUnitsInCylinder(x, z, BLOOD_MAGIC_RANGE)
	for i = 1, #units do
		if units[i] == unitID then
			table.remove(units, i)
			break
		end
	end
	return units
end

local function DrainUnits(units)
	local total = 0
	for _, unitID in pairs(units) do
		local hp = Spring.GetUnitHealth(unitID)
		local drained = MAX_DRAIN_PER_UNIT
		if hp < drained then
			drained = hp
			hp = -1
		else
			hp = hp - drained
		end
		total = total + drained
		Spring.SetUnitHealth(unitID, hp)
		Spring.Echo("drained", unitID, drained, hp)
	end
	return total
end

local function DrainNearbyUnits(unitID)
	return DrainUnits(GetNearbyUnits(unitID))
end

local function Haste(unitID)
	local mana = DrainNearbyUnits(unitID)
	Spring.Echo("Haste! Mana: ", mana)
	-- GG.Attributes.AddEffect(unitID, "somekey?", {
	-- 	accel = 1.5,
	-- 	move = 1.5
	-- })
end

local function Fireball(unitID)
	local mana = DrainNearbyUnits(unitID)
	Spring.Echo("Fireball! Mana: ", mana)
end

local function Slow(unitID)
	local mana = DrainNearbyUnits(unitID)
	Spring.Echo("Slow! Mana: ", mana)
end

-- ID mapping as well
local spells = { Haste, Fireball, Slow }

local function UseSpell(unitID, spellID, ...)
	Spring.Echo(spellID, unitID)
	local spell = spells[spellID]
	spell(unitID, ...)
end

function gadget:Initialize()
	GG.UseSpell = UseSpell
end
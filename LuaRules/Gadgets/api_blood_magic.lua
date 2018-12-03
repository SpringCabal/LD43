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

local houseDefID = UnitDefNames["house"].id
local BLOOD_MAGIC_RANGE = 600

local FIREBALL_COST = 800
local BUFF_COST = 500
local STUN_COST = 800
local THROW_COST = 1800

local PLAYER_TEAM = 0
local ENEMY_TEAM = 1
local CASTING_TIME_SHORT = 1 * 18
local CASTING_TIME = 1 * 33
local CASTING_TIME_LONG = 1 * 33

local fireballDefID = WeaponDefNames["fireball"].id
local webDefID = WeaponDefNames["web"].id
local CEG_SPAWN = [[feature_poof_spawner]]

local function SpawnBloodEffect(unitID)
	local x, _, z, _, y = Spring.GetUnitPosition(unitID, true)
	Spring.SpawnCEG(CEG_SPAWN,
		x,y,z,
		0,0,0,
		20, 20
	)
end

local function UnNeutralUnits(units)
	for i = 1, #units do
		Spring.TransferUnit(units[i], 0, false)
	end
end

local function DrainNearbyUnits(collectorID, required)
	local toKill = {}
	local toUnNeutral = {}
	for i = 1, 20 do
		local unitID = Spring.GetUnitNearestAlly(collectorID, BLOOD_MAGIC_RANGE)
		if not unitID then
			UnNeutralUnits(toUnNeutral)
			return false
		end
		if Spring.GetUnitDefID(unitID) == houseDefID then
			Spring.TransferUnit(unitID, 2, false)
			toUnNeutral[#toUnNeutral + 1] = unitID
		else
			local hp = Spring.GetUnitHealth(unitID)
			if hp > required then
				SpawnBloodEffect(unitID)
				Spring.SetUnitHealth(unitID, hp - required)
				required = 0

				UnNeutralUnits(toUnNeutral)
				for i = 1, #toKill do
					SpawnBloodEffect(toKill[i])
					Spring.DestroyUnit(toKill[i], true)
				end

				return true
			else
				toKill[#toKill + 1] = unitID
				Spring.TransferUnit(unitID, 2, false)
				required = required - hp
			end
		end
	end

	UnNeutralUnits(toUnNeutral)
	return false
end

local function Transfusion(unitID)
	local health, maxHealth = Spring.GetUnitHealth(unitID)
	if health >= maxHealth then
		return
	end
	local manaFound = DrainNearbyUnits(unitID, maxHealth - health)
	if not manaFound then
		return
	end

	local function castFunc()
		if Spring.ValidUnitID(unitID) then
			Spring.SetUnitHealth(unitID, maxHealth)
			SpawnBloodEffect(unitID)
		end
	end
	
	local env = Spring.UnitScript.GetScriptEnv(unitID)
	Spring.UnitScript.CallAsUnit(unitID, env.script.CastAnimation, castFunc, 1, tx, tz)
	Spring.SetGameRulesParam("castingFreeze", Spring.GetGameFrame() + CASTING_TIME_SHORT)
	Spring.ClearUnitGoal(unitID)
end


local function Heartburn(unitID, tx, ty, tz)
	if tx == nil then
		return
	end

	local manaFound = DrainNearbyUnits(unitID, FIREBALL_COST)
	if not manaFound then
		return
	end

	local x, y, z = Spring.GetUnitPosition(unitID)
	local function castFunc()
		ty = Spring.GetGroundHeight(tx, tz) + 20
		y = y + 120
		local dx, dy, dz = tx - x, ty - y, tz - z
		local flyTime = 10 + math.floor(math.sqrt(dx*dx + dz*dz)/45)
		local gravity = 0.18

		local vx, vy, vz = dx/flyTime, flyTime*gravity/2 + dy/flyTime, dz/flyTime
		local proID = Spring.SpawnProjectile(fireballDefID, {
			pos = { x, y, z },
			speed = { vx, vy, vz },
			ttl = flyTime,
			owner = unitID,
			--team = PLAYER_TEAM,
		})
		Spring.SetProjectileGravity(proID, -gravity)
	end
	
	local env = Spring.UnitScript.GetScriptEnv(unitID)
	Spring.UnitScript.CallAsUnit(unitID, env.script.CastAnimation, castFunc, 2, tx, tz)
	Spring.SetGameRulesParam("castingFreeze", Spring.GetGameFrame() + CASTING_TIME)
	Spring.ClearUnitGoal(unitID)
end

local function Migraine(unitID, tx, ty, tz)
	local manaFound = DrainNearbyUnits(unitID, STUN_COST)
	if not manaFound then
		return
	end
	
	local function castFunc()
		local units = Spring.GetUnitsInCylinder(tx, tz, 250, ENEMY_TEAM)
		for i = 1, #units do
			GG.StatusEffects.Stun(units[i], 220 + math.random()*30)
		end
	end
	
	local env = Spring.UnitScript.GetScriptEnv(unitID)
	Spring.UnitScript.CallAsUnit(unitID, env.script.CastAnimation, castFunc, 2, tx, tz)
	Spring.SetGameRulesParam("castingFreeze", Spring.GetGameFrame() + CASTING_TIME)
	Spring.ClearUnitGoal(unitID)
end

local function Adrenaline(unitID, tx, ty, tz)
	local manaFound = DrainNearbyUnits(unitID, BUFF_COST)
	if not manaFound then
		return
	end
	local x, y, z = Spring.GetUnitPosition(unitID)
	
	local function castFunc()
		local units = Spring.GetUnitsInCylinder(x, z, 350, PLAYER_TEAM)
		for i = 1, #units do
			GG.StatusEffects.Adrenaline(units[i], 320 + math.random()*60)
		end
	end
	
	local env = Spring.UnitScript.GetScriptEnv(unitID)
	Spring.UnitScript.CallAsUnit(unitID, env.script.CastAnimation, castFunc, 1, tx, tz)
	Spring.SetGameRulesParam("castingFreeze", Spring.GetGameFrame() + CASTING_TIME_SHORT)
	Spring.ClearUnitGoal(unitID)
end

local function Dialysis(unitID, tx, ty, tz)
	local manaFound = DrainNearbyUnits(unitID, THROW_COST)
	if not manaFound then
		return
	end
	local x, y, z = Spring.GetUnitPosition(unitID)
	
	local function castFunc()
		local units = Spring.GetUnitsInCylinder(x, z, 400, ENEMY_TEAM)
		for i = 1, #units do
			local ux, uy, uz = Spring.GetUnitPosition(units[i])
			local dx, dz = ux - x, uz - z
			local dist = math.sqrt(dx*dx + dz*dz)
			local impulse = 32*(1 - dist/800)
			Spring.AddUnitImpulse(units[i], impulse*dx/dist, impulse, impulse*dz/dist)
		end
	end
	
	local env = Spring.UnitScript.GetScriptEnv(unitID)
	Spring.UnitScript.CallAsUnit(unitID, env.script.CastAnimation, castFunc, 2, tx, tz)
	Spring.SetGameRulesParam("castingFreeze", Spring.GetGameFrame() + CASTING_TIME)
	Spring.ClearUnitGoal(unitID)
end

-- ID mapping as well
local spells = { Transfusion, Heartburn, Migraine, Adrenaline, Dialysis }

local function UseSpell(unitID, spellID, tx, ty, tz)
	Spring.Echo('Casting spell: ', spellID, unitID)
	local spell = spells[spellID]
	spell(unitID, tx, ty, tz)
end

function gadget:Initialize()
	GG.UseSpell = UseSpell
end
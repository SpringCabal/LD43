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

local FIREBALL_COST = 1000
local BUFF_COST = 900
local STUN_COST = 1200
local THROW_COST = 1800

local PLAYER_TEAM = 0
local ENEMY_TEAM = 1
local CASTING_TIME_SHORT = 1 * 18
local CASTING_TIME = 1 * 20
local CASTING_TIME_LONG = 1 * 33

local alliesDrained = 0

local fireballDefID = WeaponDefNames["fireball"].id

local CEG_FLAMES = [[feature_poof_spawner]]
local CEG_MIGRAINE = [[migraine_pulse_spawner]]
local CEG_ADRENALINE = [[adrenaline_sparkles]]

local function SpawnBloodEffect(unitID)
	local x, _, z, _, y = Spring.GetUnitPosition(unitID, true)
	Spring.SpawnCEG(CEG_FLAMES,
		x,y,z,
		0,0,0,
		20, 20
	)
end

local function SpawnMigraineEffect(x, y, z)
	Spring.SpawnCEG(CEG_MIGRAINE,
		x,y + 50,z,
		0,0,0,
		20, 20
	)
end

local function SpawnAdrenalineEffect(x, y, z)
	Spring.SpawnCEG(CEG_ADRENALINE,
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

local function DoPartial(toKill, required)
	if #toKill == 0 then
		return false
	end

	for i = 1, #toKill do
		SpawnBloodEffect(toKill[i])
		Spring.DestroyUnit(toKill[i], true)
		alliesDrained = alliesDrained + 1
		Spring.SetGameRulesParam("alliesDrained", alliesDrained)
	end

	return required
end

local function DrainNearbyUnits(collectorID, required, canPartial)
	local toKill = {}
	local toUnNeutral = {}
	local total = 0
	for i = 1, 20 do
		local unitID = Spring.GetUnitNearestAlly(collectorID, BLOOD_MAGIC_RANGE)
		if not unitID then
			UnNeutralUnits(toUnNeutral)
			if canPartial then
				return false, DoPartial(toKill, required)
			end
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
				for j = 1, #toKill do
					SpawnBloodEffect(toKill[j])
					Spring.DestroyUnit(toKill[j], true)
					alliesDrained = alliesDrained + 1
					Spring.SetGameRulesParam("alliesDrained", alliesDrained)
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
	if canPartial then
		return false, DoPartial()
	end
	return false
end

local function Transfusion(unitID)
	local health, maxHealth = Spring.GetUnitHealth(unitID)
	if health >= maxHealth then
		return
	end
	local manaFound, partialShortBy = DrainNearbyUnits(unitID, maxHealth - health, true)
	if not (manaFound or partialShortBy) then
		return
	end

	local function castFunc()
		if Spring.ValidUnitID(unitID) then
			Spring.SetUnitHealth(unitID, maxHealth - (partialShortBy or 0))
			SpawnBloodEffect(unitID)
		end
	end

	local env = Spring.UnitScript.GetScriptEnv(unitID)
	Spring.UnitScript.CallAsUnit(unitID, env.script.CastAnimation, castFunc, 1, tx, tz)
	local frame = Spring.GetGameFrame()
	Spring.SetGameRulesParam("castingFreeze", frame + CASTING_TIME_SHORT)
	Spring.SetUnitRulesParam(unitID, "start_cast", frame)
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
	local frame = Spring.GetGameFrame()
	Spring.SetGameRulesParam("castingFreeze", frame + CASTING_TIME)
	Spring.SetUnitRulesParam(unitID, "start_cast", frame)
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
			if Spring.GetUnitDefID(units[i]) ~= houseDefID then
				GG.StatusEffects.Adrenaline(units[i], 320 + math.random()*60)
				local ux, uy, uz = Spring.GetUnitPosition(units[i])
				SpawnAdrenalineEffect(ux, uy, uz)
			end
		end
		SpawnAdrenalineEffect(x, y, z)
	end

	local env = Spring.UnitScript.GetScriptEnv(unitID)
	Spring.UnitScript.CallAsUnit(unitID, env.script.CastAnimation, castFunc, 1, tx, tz)
	local frame = Spring.GetGameFrame()
	Spring.SetGameRulesParam("castingFreeze", frame + CASTING_TIME_SHORT)
	Spring.SetUnitRulesParam(unitID, "start_cast", frame)
	Spring.ClearUnitGoal(unitID)
end

local function Migraine(unitID, tx, ty, tz)
	local manaFound = DrainNearbyUnits(unitID, STUN_COST)
	if not manaFound then
		return
	end
	
	local function castFunc()
		local units = Spring.GetUnitsInCylinder(tx, tz, 300, ENEMY_TEAM)
		for i = 1, #units do
			GG.StatusEffects.Stun(units[i], 320 + math.random()*80)
		end
		SpawnMigraineEffect(tx, ty, tz)
	end
	
	local env = Spring.UnitScript.GetScriptEnv(unitID)
	Spring.UnitScript.CallAsUnit(unitID, env.script.CastAnimation, castFunc, 3, tx, tz)
	local frame = Spring.GetGameFrame()
	Spring.SetGameRulesParam("castingFreeze", frame + CASTING_TIME_LONG)
	Spring.SetUnitRulesParam(unitID, "start_cast", frame)
	Spring.ClearUnitGoal(unitID)
end

local function Dialysis(unitID, tx, ty, tz)
	local manaFound = DrainNearbyUnits(unitID, THROW_COST)
	if not manaFound then
		return
	end
	local x, y, z = Spring.GetUnitPosition(unitID)

	local function castFunc()
		local units = Spring.GetUnitsInCylinder(x, z, 650, ENEMY_TEAM)
		for i = 1, #units do
			local health = Spring.GetUnitHealth(units[i])
			Spring.SetUnitHealth(units[i], math.max(health - 2000, health/2))
			
			local ux, uy, uz = Spring.GetUnitPosition(units[i])
			local dx, dz = ux - x, uz - z
			local dist = math.sqrt(dx*dx + dz*dz)
			local impulse = 36*(1 - dist/1000)
			Spring.AddUnitImpulse(units[i], impulse*dx/dist, impulse, impulse*dz/dist)
		end
	end

	local env = Spring.UnitScript.GetScriptEnv(unitID)
	Spring.UnitScript.CallAsUnit(unitID, env.script.CastAnimation, castFunc, 3, tx, tz)
	local frame = Spring.GetGameFrame()
	Spring.SetGameRulesParam("castingFreeze", frame + CASTING_TIME_LONG)
	Spring.SetUnitRulesParam(unitID, "start_cast", frame)
	Spring.ClearUnitGoal(unitID)
end

-- ID mapping as well
local spells = { Transfusion, Heartburn, Adrenaline, Migraine, Dialysis }

local function UseSpell(unitID, spellID, tx, ty, tz)
	Spring.Echo('Casting spell: ', spellID, unitID)
	local spell = spells[spellID]
	spell(unitID, tx, ty, tz)
end

function gadget:Initialize()
	GG.UseSpell = UseSpell
end
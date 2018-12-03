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

local FIREBALL_COST = 1300
local BUFF_COST = 1100
local STUN_COST = 1300
local THROW_COST = 1750

local LIFE_BONUS = 250

local PLAYER_TEAM = 0
local ENEMY_TEAM = 1
local CASTING_TIME_SHORT = 1 * 8
local CASTING_TIME = 1 * 12
local CASTING_TIME_LONG = 1 * 25

local alliesDrained = 0

local fireballDefID = WeaponDefNames["fireball"].id

local CEG_SINGLE_BLOOD = [[feature_poof_single]]
local CEG_FLAMES = [[feature_poof_spawner]]
local CEG_HEALTH = [[feature_slurp_spawner]]
local CEG_MIGRAINE = [[migraine_pulse_spawner]]
local CEG_ADRENALINE = [[adrenaline_sparkles]]
local CEG_DIALYSIS = [[gtfo_pulse]]

local SOUND_DRAIN = "sounds/blooddrain.wav"

local function SpawnEffect(unitID, cegName, sound)
	local x, _, z, _, y = Spring.GetUnitPosition(unitID, true)
	if not x then
		return
	end
	Spring.SpawnCEG(cegName,
		x,y,z,
		0,0,0,
		20, 20
	)
	if sound then
		GG.PlaySound(sound, 10, x, y, z)
	end
end

local function SpawnEffectPosition(x, y, z, cegName)
	Spring.SpawnCEG(cegName,
		x,y,z,
		0,0,0,
		20, 20
	)
end

local function MakeBloodTrail(x, y, z, targetID)
	local tx, _, tz, _, ty = Spring.GetUnitPosition(targetID, true)
	if not tx then
		return
	end
	local dx, dy, dz = x - tx, y - ty, z - tz
	local dist = math.max(0.1, math.sqrt(dx*dx + dy*dy + dz*dz))
	dx, dy, dz = dx/dist, dy/dist, dz/dist
	
	for i = 10, dist, 10 do
		SpawnEffectPosition(tx + dx*i, ty + dy*i, tz + dz*i, CEG_SINGLE_BLOOD)
	end
end

local function UnNeutralUnits(units)
	for i = 1, #units do
		Spring.TransferUnit(units[i], 0, false)
	end
end

local function DoPartial(toKill, required, x, y, z)
	if #toKill == 0 then
		return false
	end

	for i = 1, #toKill do
		SpawnEffect(toKill[i], CEG_FLAMES, SOUND_DRAIN)
		MakeBloodTrail(x, y, z, toKill[i])
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
	local x, _, z, _, y = Spring.GetUnitPosition(collectorID, true)
	for i = 1, 50 do
		local unitID = Spring.GetUnitNearestAlly(collectorID, BLOOD_MAGIC_RANGE)
		if not unitID then
			UnNeutralUnits(toUnNeutral)
			if canPartial then
				return false, DoPartial(toKill, required, x, y, z)
			end
			return false
		end
		if Spring.GetUnitDefID(unitID) == houseDefID then
			Spring.TransferUnit(unitID, 2, false)
			toUnNeutral[#toUnNeutral + 1] = unitID
		else
			local hp = Spring.GetUnitHealth(unitID) + LIFE_BONUS
			if hp > required then
				SpawnEffect(unitID, CEG_FLAMES, SOUND_DRAIN)
				MakeBloodTrail(x, y, z, unitID)
				Spring.SetUnitHealth(unitID, hp - required)
				required = 0

				UnNeutralUnits(toUnNeutral)
				for j = 1, #toKill do
					SpawnEffect(toKill[j], CEG_FLAMES, SOUND_DRAIN)
					MakeBloodTrail(x, y, z, toKill[j])
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
	local manaFound, partialShortBy = DrainNearbyUnits(unitID, LIFE_BONUS + maxHealth - health, true)
	if not (manaFound or partialShortBy) then
		return
	end
	if partialShortBy then
		partialShortBy = partialShortBy - LIFE_BONUS
	end

	local function castFunc()
		if Spring.ValidUnitID(unitID) then
			Spring.SetUnitHealth(unitID, maxHealth - (partialShortBy or 0))
			SpawnEffect(unitID, CEG_HEALTH)
		end
	end

	local x, _, z, _, y = Spring.GetUnitPosition(unitID, true)
	GG.PlaySound("sounds/heal_spell.wav", 10, x, y, z)
	
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
		local units = Spring.GetUnitsInCylinder(x, z, 450, PLAYER_TEAM)
		for i = 1, #units do
			if Spring.GetUnitDefID(units[i]) ~= houseDefID then
				GG.StatusEffects.Adrenaline(units[i], 320 + math.random()*60)
				local ux, uy, uz = Spring.GetUnitPosition(units[i])
				SpawnEffectPosition(ux, uy, uz, CEG_ADRENALINE)
			end
		end
		SpawnEffectPosition(x, y, z, CEG_ADRENALINE)
	end
	GG.PlaySound("sounds/buff_spell.wav", 10, x, y, z)

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
		local units = Spring.GetUnitsInCylinder(tx, tz, 450, ENEMY_TEAM)
		for i = 1, #units do
			local ux, uy, uz = Spring.GetUnitPosition(units[i])
			local dx, dz = ux - tx, uz - tz
			local dist = math.sqrt(dx*dx + dz*dz)
			GG.StatusEffects.Stun(units[i], (1 - dist/1000)*(400 + math.random()*80))
		end
		SpawnEffectPosition(tx, ty, tz, CEG_MIGRAINE)
	end
	GG.PlaySound("sounds/curse_spell.wav", 10, tx, ty, tz)
	
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
		local units = Spring.GetUnitsInCylinder(x, z, 700, ENEMY_TEAM)
		for i = 1, #units do
			local health = Spring.GetUnitHealth(units[i])
			Spring.SetUnitHealth(units[i], math.max(health - 1500, health/2))
			
			local ux, uy, uz = Spring.GetUnitPosition(units[i])
			local dx, dz = ux - x, uz - z
			local dist = math.sqrt(dx*dx + dz*dz)
			local impulse = 36*(1 - dist/1000)
			Spring.AddUnitImpulse(units[i], impulse*dx/dist, impulse, impulse*dz/dist)
		end
		SpawnEffectPosition(x, y, z, CEG_DIALYSIS)
	end
	GG.PlaySound("sounds/throw_spell.wav", 10, x, y, z)

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
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
local BLOOD_MAGIC_RANGE = 500
local MAX_DRAIN_PER_UNIT = 10000
local FIREBALL_COST = 700

local PLAYER_TEAM = 0
local CASTING_TIME = 1 * 33

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

local function DrainNearbyUnits(collectorID, required)
	local toKill = {}
	local toUnNeutral = {}
	for i = 1, 20 do
		local unitID = Spring.GetUnitNearestAlly(collectorID, BLOOD_MAGIC_RANGE)
		if not unitID then
			return false
		end
		if Spring.GetUnitDefID(unitID) == houseDefID then
			Spring.SetUnitNeutral(unitID, true)
			toUnNeutral[#toUnNeutral + 1] = unitID
		else
			local hp = Spring.GetUnitHealth(unitID)
			if hp > required then
				SpawnBloodEffect(unitID)
				Spring.SetUnitHealth(unitID, hp - required)
				required = 0

				for i = 1, #toKill do
					SpawnBloodEffect(toKill[i])
					Spring.DestroyUnit(toKill[i], true)
				end

				for i = 1, #toUnNeutral do
					Spring.SetUnitNeutral(toUnNeutral[i], false)
				end

				return true
			else
				toKill[#toKill + 1] = unitID
				required = required - hp
			end
		end
	end

	return false
end

local function Haste(unitID)
	local manaFound = DrainNearbyUnits(unitID, 200)
	if not manaFound then
		return
	end
	-- GG.Attributes.AddEffect(unitID, "somekey?", {
	-- 	accel = 1.5,
	-- 	move = 1.5
	-- })

	return true
end


local function Fireball(unitID, tx, ty, tz)
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
		y = y + 20
		local dx, dy, dz = tx - x, ty - y, tz - z
		local flyTime = 10 + math.floor(math.sqrt(dx*dx + dz*dz)/45)
		local gravity = 0.2

		local vx, vy, vz = dx/flyTime, flyTime*gravity/2 + dy/flyTime, dz/flyTime
		local proID = Spring.SpawnProjectile(fireballDefID, {
			pos = { x, y, z },
			speed = { vx, vy, vz },
			ttl = flyTime,
			owner = unitID,
			team = PLAYER_TEAM,
		})
		Spring.SetProjectileGravity(proID, -gravity)
	end
	
	local env = Spring.UnitScript.GetScriptEnv(unitID)
	Spring.UnitScript.CallAsUnit(unitID, env.script.CastAnimation, castFunc, tx, tz)
	Spring.SetGameRulesParam("castingFreeze", Spring.GetGameFrame() + CASTING_TIME)
	Spring.ClearUnitGoal(unitID)
	
	return true
end

local function Slow(unitID, tx, ty, tz)
	local manaFound = DrainNearbyUnits(unitID, FIREBALL_COST)
	if not manaFound then
		return
	end


	ty = Spring.GetGroundHeight(tx, tz)
	local x, y, z = Spring.GetUnitPosition(unitID)
	local vx = tx - x
	local vy = ty - y
	local vz = tz - z
	local d = math.sqrt(vx*vx + vy*vy + vz*vz)
	local SPEED = 40
	local multiplier = SPEED / (d + 0.0001)
	vx = vx * multiplier
	vy = vy * multiplier
	vz = vz * multiplier
	Spring.SpawnProjectile(webDefID, {
		pos = { x, y, z },
		speed = { vx, vy, vz },
		ttl = 30,
		-- ["end"] = { tx, ty, tz },
		owner = unitID,
	})

	return true
end

-- ID mapping as well
local spells = { Haste, Fireball, Slow }

local function UseSpell(unitID, spellID, tx, ty, tz)
	Spring.Echo('Casting spell: ', spellID, unitID)
	local spell = spells[spellID]
	spell(unitID, tx, ty, tz)
end

function gadget:Initialize()
	GG.UseSpell = UseSpell
end
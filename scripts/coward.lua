local Torso = piece("Torso")


function script.QueryWeapon()
	return Torso
end

function script.AimWeapon()
	return true
end

function script.StartBuilding(heading, pitch)
	SetUnitValue(COB.INBUILDSTANCE, 1)
end

local shared = include("shared.lua")
function script.Create()
	shared.Init(Torso, 3)
	shared.InitSound("sounds/panicdeathyell.wav", 0.6)
end

function script.StartMoving()
	shared.StartMoving()
end

function script.StopMoving()
	shared.StopMoving()
end

function script.BlockShot(num, targetID)
	if not targetID then
		return true
	end
	local tx, _, tz = Spring.GetUnitPosition(targetID)
	local ux, uy, uz = Spring.GetUnitPosition(unitID)
	local dx, dz = tx - ux, tz - uz
	local cx = ux - dx
	local cz = uz - dz
	local cy = Spring.GetGroundHeight(cx, cz)
	
	if deathSound and math.random() < 0.35 then
		local x, y, z = Spring.GetUnitPosition(unitID)
		if x then
			GG.PlaySound("sounds/runawayyell.wav", 10, ux, uy, uz)
		end
	end
	
	Spring.GiveOrderToUnit(unitID, CMD.MOVE, {cx, cy, cz}, {})
	
	Spring.SetUnitWeaponState(unitID, 1, "reloadFrame", Spring.GetGameFrame() + 180)
	return true
end
local House = piece("House")

function script.HitByWeapon(x, z, weaponID, damage)
	if damage > 400 then
		Spring.SetUnitRulesParam(unitID, "on_fire", 1)
	end
end

function script.Killed(recentDamage, maxHealth)
	Spring.SetUnitCollisionVolumeData(unitID, 5, 5, 5, 0, 0, 0, 0, 1, 0)
	Spring.SetUnitSelectionVolumeData(unitID, 5, 5, 5, 0, 0, 0, -1, 1, 0)
	Spring.SetUnitNeutral(unitID, true)

	
	local x, y, z = Spring.GetUnitPosition(unitID)
	if x then
		GG.PlaySound("sounds/house_fall.wav", 10, x, y, z)
	end
	
	Move(House, z_axis, -250, 250)
	Turn(House, x_axis, (math.random() - 0.5)/2, math.random()/2)
	Turn(House, y_axis, (math.random() - 0.5)/2, math.random()/2)
	Turn(House, z_axis, (math.random() - 0.5)/4, math.random()/4)
	Sleep(1200)
	return 0
end

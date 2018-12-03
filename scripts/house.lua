local House = piece("House")

function script.HitByWeapon(x, z, weaponID, damage)
	if damage > 400 then
		Spring.SetUnitRulesParam(unitID, "on_fire", 1)
	end
end

function script.Killed(recentDamage, maxHealth)
	Move(House, z_axis, -500, 700)
	Turn(House, x_axis, math.random() - 0.5, math.random())
	Turn(House, y_axis, math.random() - 0.5, math.random())
	Turn(House, z_axis, (math.random() - 0.5)/2, math.random()/2)
	Sleep(1000)
	return 0
end


function script.HitByWeapon(x, z, weaponID, damage)
	if damage > 400 then
		Spring.SetUnitRulesParam(unitID, "on_fire", 1)
	end
end
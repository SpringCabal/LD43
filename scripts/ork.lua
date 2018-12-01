local handRight = piece("Hand_Right")

function script.QueryWeapon()
    Spring.Echo("QUERY")
    return handRight
end

function script.AimWeapon()
    Spring.Echo("AIM")
    return true
end
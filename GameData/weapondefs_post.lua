--------------------------------------------------------------------------------
-- The Def:New/Clone/Append helpers from weapondefs_pre.lua are inherited into
-- every def table by value.  lowerkeys() then turns them into "new"/"clone"/
-- "append" tags, which the engine reports as unknown.  Strip every function
-- value before the defs reach the engine.
--------------------------------------------------------------------------------

local function StripFunctions(t, seen)
	if seen[t] then
		return
	end
	seen[t] = true
	for k, v in pairs(t) do
		if type(v) == "function" then
			t[k] = nil
		elseif type(v) == "table" then
			StripFunctions(v, seen)
		end
	end
end

for _, wd in pairs(WeaponDefs) do
	StripFunctions(wd, {})
	-- The guest WeaponDefInfo API omits visual size/color. Preserve the
	-- definition inputs as custom params for the Lua-equivalent light pass.
	wd.customparams = wd.customparams or {}
	local cp = wd.customparams
	cp.projectile_rgb = wd.rgbcolor or "1 0.5 0"
	if wd.size then
		cp.projectile_size = wd.size
	end
end

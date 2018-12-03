local function RecursiveReplaceStrings(t, name, replacedMap)
	if (replacedMap[t]) then
		return  -- avoid recursion / repetition
	end
	replacedMap[t] = true
	local changes = {}
	for k, v in pairs(t) do
		if (type(v) == 'string') then
			t[k] = v:gsub("<NAME>", name)
		end
		if (type(v) == 'table') then
			RecursiveReplaceStrings(v, name, replacedMap)
		end
	end 
end

local function ReplaceStrings(t, name)
	local replacedMap = {}
	RecursiveReplaceStrings(t, name, replacedMap)
end

-- Process ALL the units!
for name, ud in pairs(UnitDefs) do
	-- Replace all occurences of <NAME> with the respective values
	ReplaceStrings(ud, ud.unitname or name)
	Spring.Echo(name)
end

-- customParams is always defined.
for name, ud in pairs(UnitDefs) do
	ud.customParams = ud.customParams or {}
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
--
-- Automatically generate some big selection volumes.
--

local function Explode(div, str)
	if div == '' then
		return false
	end
	local pos, arr = 0, {}
	-- for each divider found
	for st, sp in function() return string.find(str, div, pos, true) end do
		table.insert(arr, string.sub(str, pos, st - 1)) -- Attach chars left of current divider
		pos = sp + 1 -- Jump past current divider
	end
	table.insert(arr,string.sub(str,pos)) -- Attach chars right of last divider
	return arr
end

local function GetDimensions(scale)
	if not scale then
		return false
	end
	local dimensionsStr = Explode(" ", scale)
	-- string conversion (required for MediaWiki export)
	local dimensions = {}
	for i,v in pairs(dimensionsStr) do
		dimensions[i] = tonumber(v)
	end
	local largest = (dimensions and dimensions[1] and tonumber(dimensions[1])) or 0
	for i = 2, 3 do
		largest = math.max(largest, (dimensions and dimensions[i] and tonumber(dimensions[i])) or 0)
	end
	return dimensions, largest
end

local VISUALIZE_SELECTION_VOLUME = false
local CYL_SCALE = 1.1
local CYL_LENGTH = 0.8
local CYL_ADD = 5
local SEL_SCALE = 1.35

for name, ud in pairs(UnitDefs) do
	if ud.collisionvolumescales or ud.selectionvolumescales then
		-- Do not override default colvol because it is hard to measure.
		if ud.acceleration and ud.acceleration > 0 and ud.canmove then
			if ud.selectionvolumescales then
				local dim = GetDimensions(ud.selectionvolumescales)
				ud.selectionvolumescales  = math.ceil(dim[1]*SEL_SCALE) .. " " .. math.ceil(dim[2]*SEL_SCALE) .. " " .. math.ceil(dim[3]*SEL_SCALE)
			else
				local size = math.max(ud.footprintx or 0, ud.footprintz or 0)*15
				if size > 0 then
					local dimensions, largest = GetDimensions(ud.collisionvolumescales)
					local x, y, z = size, size, size
					if size > largest then
						ud.selectionvolumeoffsets = "0 0 0"
						ud.selectionvolumetype    = "ellipsoid"
					elseif string.lower(ud.collisionvolumetype) == "cylx" then
						ud.selectionvolumeoffsets = ud.collisionvolumeoffsets or "0 0 0"
						x = dimensions[1]*CYL_LENGTH
						y = math.max(dimensions[2], math.min(size, CYL_ADD + dimensions[2]*CYL_SCALE))
						z = math.max(dimensions[3], math.min(size, CYL_ADD + dimensions[3]*CYL_SCALE))
						ud.selectionvolumetype    = ud.collisionvolumetype
					elseif string.lower(ud.collisionvolumetype) == "cyly" then
						ud.selectionvolumeoffsets = ud.collisionvolumeoffsets or "0 0 0"
						x = math.max(dimensions[1], math.min(size, CYL_ADD + dimensions[1]*CYL_SCALE))
						y = dimensions[2]*CYL_LENGTH
						z = math.max(dimensions[3], math.min(size, CYL_ADD + dimensions[3]*CYL_SCALE))
						ud.selectionvolumetype    = ud.collisionvolumetype
					elseif string.lower(ud.collisionvolumetype) == "cylz" then
						ud.selectionvolumeoffsets = ud.collisionvolumeoffsets or "0 0 0"
						x = math.max(dimensions[1], math.min(size, CYL_ADD + dimensions[1]*CYL_SCALE))
						y = math.max(dimensions[2], math.min(size, CYL_ADD + dimensions[2]*CYL_SCALE))
						z = dimensions[3]*CYL_LENGTH
						ud.selectionvolumetype    = ud.collisionvolumetype
					elseif string.lower(ud.collisionvolumetype) == "box" then
						ud.selectionvolumeoffsets = "0 0 0"
						x = dimensions[1]
						y = dimensions[2]
						z = dimensions[3]
						ud.selectionvolumetype    = ud.collisionvolumetype
					end
					ud.selectionvolumescales  = math.ceil(x*SEL_SCALE) .. " " .. math.ceil(y*SEL_SCALE) .. " " .. math.ceil(z*SEL_SCALE)
				end
			end
		end
	end
	
	if VISUALIZE_SELECTION_VOLUME then
		if ud.selectionvolumescales then
			ud.collisionvolumeoffsets = ud.selectionvolumeoffsets
			ud.collisionvolumescales  = ud.selectionvolumescales
			ud.collisionvolumetype    = ud.selectionvolumetype
		end
	end
end

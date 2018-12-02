-- WIP
function widget:GetInfo()
	return {
		name    = 'target display',
		desc    = 'Displays target health bar',
		author  = 'gajop, Bluestone, Funkencool',
		date    = 'LD 43',
		license = 'GNU GPL v2',
        layer = 0,
		enabled = true,
	}
end


local Chili, window

local meter = {}
local lblName

-------------------------------------------

local hpcolormap      = { {1.0, 0.0, 0.0, 1.0},  {0.8, 0.60, 0.0, 1.0}, {0.0, 0.75, 0.0, 1.0} }

function GetColor(colormap,slider)
  local coln = #colormap
  if (slider>=1) then
    local col = colormap[coln]
    return col[1],col[2],col[3],col[4]
  end
  if (slider<0) then slider=0 elseif(slider>1) then slider=1 end
  local posn  = 1+(coln-1) * slider
  local iposn = math.floor(posn)
  local aa    = posn - iposn
  local ia    = 1-aa

  local col1,col2 = colormap[iposn],colormap[iposn+1]

  return col1[1]*ia + col2[1]*aa, col1[2]*ia + col2[2]*aa,
         col1[3]*ia + col2[3]*aa, col1[4]*ia + col2[4]*aa
end

-------------------------------------------

local function initWindow()
	local screen0 = Chili.Screen0

	window = Chili.Panel:New {
		parent    = screen0,
		padding   = {0,0,0,0},
	}

end

local function makeBar(name)
	-- local control = Chili.Control:New {
	-- 	parent    = window,
	-- 	name      = name,
	-- 	x         = '10%',
	-- 	y         = 0,
	-- 	height    = '100%',
	-- 	width     = '90%',
	-- 	padding   = {10,10,10,10},
	-- }

	Chili.Image:New {
		parent = window,
		-- file   = images[name],
        name   = 'heart',
		height = '100%',
		width  = '8%',
		x      = '5%',
		y      = 0,
        bottom = 0,
	}

	meter[name] = Chili.Progressbar:New{
		parent = window,
		x      = 0,
        y      = 0,
		bottom = 30,
		right  = 0,
	}

	-- not really a progerssbar but \o/
	lblName = Chili.Progressbar:New{
		parent = window,
		x      = 0,
        y      = 50,
		bottom = 0,
		right  = 0,
		color = {
			1,
			0,
			0,
			0.5,
		},
		caption = "",
	}

	meter[name].ratio = -2

end


local function resizeUI(vsx,vsy)
    window:SetPos(vsx*0.35, vsy*0.01, vsx*0.17, vsy*0.07)
end
function widget:ViewResize(vsx,vsy)
	resizeUI(vsx,vsy)
end

-- Updates
local function SetBarValue(name, value, maxValue)
	meter[name]:SetMinMax(0, maxValue)
	meter[name]:SetValue(value)
	if value <= 0 then
		meter[name]:SetCaption("DEAD")
	else
		meter[name]:SetCaption(tostring(math.floor(100 * value / maxValue)) .. "%")
	end
end
function SetBarColor(name,slider)
    local r,g,b,a = GetColor(hpcolormap,slider)
    meter[name]:SetColor(r,g,b,a)
end

-------------------------------------------
-- Callins
-------------------------------------------

local targetUnitDefID = nil
function display(unitID, name)
	if not unitID then
		if window.visible then
			window:Hide()
		end
		return
	end

	local h, mh = Spring.GetUnitHealth(unitID)
	h = math.max(0, h)

	local ratio = math.floor(100 * h/mh)
	if meter[name].ratio ~= ratio then
		Spring.Echo('diff ratio', ratio)
		SetBarValue('Health', h, mh)
		SetBarColor('Health', ratio / 100)
		meter[name].ratio = ratio
	end

	local unitDefID = Spring.GetUnitDefID(unitID)
	if targetUnitDefID ~= unitDefID then
		targetUnitDefID = unitDefID
		lblName:SetCaption(UnitDefs[unitDefID].humanName)
	end

	if window.hidden then
		window:Show()
	end
end

function widget:Update()
	local mx, my, lmb, mmb, rmb = Spring.GetMouseState()
	local traceType, unitID = Spring.TraceScreenRay(mx, my)
	if traceType ~= "unit" then
		display(nil, "Health")
		return
	end
	display(unitID, "Health")
end

function widget:Initialize()
	Chili = WG.Chili
	initWindow()
	makeBar('Health')
	local vsx,vsy = Spring.GetViewGeometry()
	resizeUI(vsx,vsy)
end

function widget:Shutdown()
	if window then
		window:Dispose()
	end
end

function widget:GetInfo()
  return {
    name      = "Chili Minimap",
    desc      = "v0.895 Chili Minimap",
    author    = "Licho, CarRepairer",
    date      = "@2010",
    license   = "GNU GPL, v2 or later",
    layer     = -100000,
    enabled   = true,
  }
end

--// gl const

local GL_DEPTH_BITS = 0x0D56

local GL_DEPTH_COMPONENT   = 0x1902
local GL_DEPTH_COMPONENT16 = 0x81A5
local GL_DEPTH_COMPONENT24 = 0x81A6
local GL_DEPTH_COMPONENT32 = 0x81A7

local GL_COLOR_ATTACHMENT0_EXT = 0x8CE0
local GL_COLOR_ATTACHMENT1_EXT = 0x8CE1
local GL_COLOR_ATTACHMENT2_EXT = 0x8CE2
local GL_COLOR_ATTACHMENT3_EXT = 0x8CE3

--// gl vars
 
local fbo
local offscreentex

local fadeShader
local alphaLoc
local boundsLoc

--//

local window
local fakewindow
local map_panel 
local buttons_panel
local Chili
local glDrawMiniMap = gl.DrawMiniMap
local glResetState = gl.ResetState
local glResetMatrices = gl.ResetMatrices
local echo = Spring.Echo

local iconsize = 20
local bgColor_panel = {nil, nil, nil, 0.0}
local final_opacity = 0
local last_alpha = 1 --Last set alpha value for the actual clickable minimap image
local default_fog_brightness = 0.5

local tabbedMode = false

local usingNewEngine = (#{Spring.GetLosViewColors()} == 5) -- newer engine has radar2
--local init = true

WG.MinimapDraggingCamera = false --Boolean, false if selection through minimap is possible

local function toggleTeamColors()
	if WG.LocalColor and WG.LocalColor.localTeamColorToggle then
		WG.LocalColor.localTeamColorToggle()
	else
		Spring.SendCommands("luaui enablewidget Local Team Colors")
	end
end 

local mapRatio = Game.mapX/Game.mapY
local mapIsWider = mapRatio > 1
local function AdjustToMapAspectRatio(w, h, buttonRight)
	local wPad = 16
	local hPad = 16
	if buttonRight then
		wPad = wPad + iconsize*1.3
	else
		hPad = hPad + iconsize*1.3
	end
	w = w - wPad
	h = h - hPad
	if w/h < mapRatio then
		return w + wPad, w/mapRatio + hPad
	end
	return math.ceil(h*mapRatio + wPad), math.ceil(h + hPad)
end

local function AdjustMapAspectRatioToWindow(x,y,w,h)
	local newW, newH = w,h
	local newX, newY = x,y
	if w/h > mapRatio then
		newW = mapRatio*h
		newX = x + (w-newW)/2
	else
		newH = w/mapRatio
		newY = y + (h-newH)/2
	end
	return newX, newY, newW, newH
end

local function MakeMinimapWindow()
end

options_path = 'Settings/Interface/Map'
local minimap_path = 'Settings/HUD Panels/Minimap'
local hotkeysPath = 'Hotkeys/Misc'
--local radar_path = 'Settings/Interface/Map/Radar View Colors'
local radar_path = 'Settings/Interface/Map/Radar Color'
local radar_path_edit = 'Settings/Interface/Map/Radar Color'
options_order = { 
	'label_drawing', 
	'drawinmap',
	'clearmapmarks', 
	'lastmsgpos',
	
	'lblViews', 
	'viewstandard', 
	'viewheightmap', 
	'viewblockmap', 
	'viewfow',
	'showeco',
	
	'lable_initialView', 
	'initialSensorState',
	
	-- Radar view configuration
	'radar_view_colors_label1',
	'radar_fog_brightness1', 
	
	-- Radar view editing
	'radar_view_colors_label2', 
	'radar_radar_color', 
	'radar_radar2_color',
	'radar_jammer_color', 

	-- Radar view presets
	'radar_view_presets_label1',
	'radar_preset_only_los', 
	'radar_preset_double_outline', 
	'radar_preset_blue_line',  
	'radar_preset_green', 
	'radar_preset_green_in_blue', 
	
	-- Minimap options
	'disableMinimap',
	'hideOnOverview',
	'use_map_ratio',
	'opacity', 
	'alwaysResizable', 
	'buttonsOnRight', 
	'hidebuttons', 
	'minimizable',
	'lblblank1',

	'leftClickOnMinimap', 
	'fadeMinimapOnZoomOut', 
	
	'fancySkinning',
}
options = {
	label_drawing = { type = 'label', name = 'Map Drawing and Messaging', path = hotkeysPath},
	
	drawinmap = {
		name = 'Map Drawing Hotkey',
		desc = 'Hold this hotkey to draw on the map and write messages. Left click to draw, right click to erase, middle click to place a marker. Double left click to type a marker message.',
		type = 'button',
		action = 'drawinmap',
		path = hotkeysPath,
	},
	clearmapmarks = {
		name = 'Erase Map Drawing',
		desc = 'Erases all map drawing and markers (for you, not for others on your team).',
		type = 'button',
		action = 'clearmapmarks',
		path = hotkeysPath,
	},	
	lastmsgpos = {
		name = 'Zoom To Last Message',
		desc = 'Moves the camera to the most recently placed map marker or message.',
		type = 'button',
		action = 'lastmsgpos',
		path = hotkeysPath,
	},	
	
	lblViews = { type = 'label', name = 'Map Overlays', path = hotkeysPath},

	viewstandard = {
		name = 'Clear Overlays',
		desc = 'Disables Heightmap, Pathing and Line of Sight overlays.',
		type = 'button',
		action = 'showstandard',
		path = hotkeysPath,
	},
	viewheightmap = {
		name = 'Toggle Height Map',
		desc = 'Shows contours of terrain elevation.',
		type = 'button',
		action = 'showelevation',
		path = hotkeysPath,
	},
	viewblockmap = {
		name = 'Toggle Pathing Map',
		desc = 'Select a unit to see where it can go. Select a building blueprint to see where it can be placed.',
		type = 'button',
		action = 'showpathtraversability',
		path = hotkeysPath,
	},
	
	viewfow = {
		name = 'Toggle Line of Sight',
		desc = 'Shows sight distance and radar coverage.',
		type = 'button',
		action = 'togglelos',
		path = hotkeysPath,
	},
	
	showeco = {
		name = 'Toggle Economy Overlay',
		desc = 'Show metal, geo spots and energy grid',
		hotkey = {key='f4', mod=''},
		type ='button',
		action='showeco',
		noAutoControlFunc = true,
		OnChange = function(self)
			if (WG.ToggleShoweco) then
				WG.ToggleShoweco()
			end
		end,
		path = hotkeysPath,
	},
	
	lable_initialView = { type = 'label', name = 'Initial Map Overlay', },
	
	initialSensorState = {
		name = "Start with LOS enabled",
		desc = "Game starts with Line of Sight Overlay enabled",
		type = 'bool',
		value = true,
		noHotkey = true,
	},
	
--------------------------------------------------------------------------
-- Configure Radar and Line of Sight 'Settings/Interface/Map/Radar'
--------------------------------------------------------------------------	
	
	radar_view_colors_label1 = { 
		type = 'label', name = 'Other Options',
	},
	
	radar_fog_brightness1 = {
		name = "Fog Brightness",
		type = "number",
		value = default_fog_brightness, min = 0, max = 1, step = 0.01,
		OnChange =  function() updateRadarColors() end,
		path = radar_path,
	},
	
--------------------------------------------------------------------------
-- Radar view color editing 'Settings/Interface/Map/Radar'
--------------------------------------------------------------------------
	
	radar_view_colors_label2 = { 
		type = 'label', name = '* Note: These colors are additive.', path = radar_path_edit,
	},

	radar_radar_color = {
		name = "Radar Edge Color",
		type = "colors",
		value = { 0, 0, 1, 0},
		OnChange =  function() updateRadarColors() end,
		path = radar_path_edit,
	},
	radar_radar2_color = {
		name = "Radar Interior Color",
		type = "colors",
		value = { 0, 1, 0, 0},
		OnChange =  function() updateRadarColors() end,
		path = radar_path_edit,
	},
	radar_jammer_color = {
		name = "Jammer Color",
		type = "colors",
		value = { 0.1, 0, 0, 0},
		OnChange = function() updateRadarColors() end,
		path = radar_path_edit,
	},
	
--------------------------------------------------------------------------
-- Radar view presets 'Settings/Interface/Map/Radar'
--------------------------------------------------------------------------

	radar_view_presets_label1 = {
		type = 'label', name = 'Radar Presets', path = radar_path,
	},

	radar_preset_only_los = {
		name = 'Only LOS',
		type = 'button',
		OnChange = function()
			-- options.radar_fog_color.value = { 0.25, 0.25, 0.25, 1}
			-- options.radar_los_color.value = { 0.25, 0.25, 0.25, 1}
			options.radar_fog_brightness1.value = default_fog_brightness
			options.radar_radar_color.value = { 0, 0, 0, 0}
			options.radar_radar2_color.value = { 0, 0, 0, 0}
			options.radar_jammer_color.value = { 0, 0, 0, 0}
			updateRadarColors()
			WG.crude.OpenPath(radar_path, false)
		end,
		path = radar_path,
	},
	
	radar_preset_double_outline = {
		name = 'Double Outline (default)',
		type = 'button',
		OnChange = function()
			options.radar_fog_brightness1.value = default_fog_brightness
			options.radar_jammer_color.value = { 0.1, 0, 0, 0}
			options.radar_radar_color.value = { 0, 0, 1, 0}
			options.radar_radar2_color.value = { 0, 1, 0, 0}

			updateRadarColors()
			WG.crude.OpenPath(radar_path, false)
		end,
		path = radar_path,
	},
	radar_preset_blue_line = {
		name = 'Blue Outline',
		type = 'button',
		OnChange = function()
			options.radar_fog_brightness1.value = default_fog_brightness
			options.radar_jammer_color.value = { 0.1, 0, 0, 0}
			options.radar_radar_color.value = { 0, 0, 1, 0}
			options.radar_radar2_color.value = { 0, 0, 1, 0}
			updateRadarColors()
			WG.crude.OpenPath(radar_path, false)
		end,
		path = radar_path,
	},
	
	radar_preset_green = {
		name = 'Green Area Fill',
		type = 'button',
		OnChange = function()
			options.radar_fog_brightness1.value = default_fog_brightness
			options.radar_radar_color.value = { 0, 0.17, 0, 0}
			options.radar_radar2_color.value = { 0, 0.17, 0, 0}
			options.radar_jammer_color.value = { 0.18, 0, 0, 0}
			updateRadarColors()
			WG.crude.OpenPath(radar_path, false)
		end,
		path = radar_path,
	},
	
	radar_preset_green_in_blue = {
		name = 'Green in Blue Outline',
		type = 'button',
		OnChange = function()
			options.radar_fog_brightness1.value = default_fog_brightness
			options.radar_radar_color.value = { 0, 0, 0.4, 0}
			options.radar_radar2_color.value = { 0, 0.04, 1, 0}
			options.radar_jammer_color.value = { 0.18, 0, 0, 0}
			updateRadarColors()
			WG.crude.OpenPath(radar_path, false)
		end,
		path = radar_path,
	},


--------------------------------------------------------------------------
-- Minimap path area 'Settings/HUD Panels/Minimap'
--------------------------------------------------------------------------
	disableMinimap = {
		name = 'Disable Minimap',
		type = 'bool',
		value = false,
		OnChange = function(self) MakeMinimapWindow() end,
		path = minimap_path,
	},
	hideOnOverview = {
		name = 'Hide on Overview',
		type = 'bool',
		value = false,
		OnChange = function(self) MakeMinimapWindow() end,
		path = minimap_path,
		noHotkey = true,
	},	
	use_map_ratio = {
		name = 'Keep Aspect Ratio',
		type = 'radioButton',
		value = 'arwindow',
		items = {
			{key = 'arwindow',  name = 'Aspect Ratio Window'},
			{key = 'armap',     name = 'Aspect Ratio Map'},
			{key = 'arnone',    name = 'Map Fills Window'},
		},
		OnChange = function(self)
			local arwindow = self.value == 'arwindow'
			window.fixedRatio = arwindow
			if arwindow then
				local maxSize = math.max(window.width, window.height)
				local w,h = AdjustToMapAspectRatio(maxSize, maxSize, options.buttonsOnRight.value)
				window:Resize(w,h,false,false)
			end 
		end,
		path = minimap_path,
		noHotkey = true,
	},	
	opacity = {
		name = "Opacity",
		type = "number",
		value = 0, min = 0, max = 1, step = 0.01,
		OnChange = function(self)
			if self.value == 0 then
				bgColor_panel = {nil, nil, nil, 1}
			else
				bgColor_panel = {nil, nil, nil, 0}
			end
			final_opacity = self.value * last_alpha
			last_alpha = 2 --invalidate last_alpha so it needs to be recomputed
			MakeMinimapWindow()
			window:Invalidate()
		end,
		path = minimap_path,
	},
	alwaysResizable = {
		name = 'Resizable',
		type = 'bool',
		value = false,
		OnChange= function(self) MakeMinimapWindow() end,
		path = minimap_path,
		noHotkey = true,
	},	
	buttonsOnRight = {
		name = 'Map buttons on the right',
		type = 'bool',
		value = false,
		OnChange = function(self) MakeMinimapWindow() end,
		path = minimap_path,
		noHotkey = true,
	},
	hidebuttons = {
		name = 'Hide Minimap Buttons',
		type = 'bool',
		advanced = true,
		OnChange= function(self)
			iconsize = self.value and 0 or 20 
			MakeMinimapWindow() 
		end,
		value = false,
		path = minimap_path,
		noHotkey = true,
	},
	minimizable = {
		name = 'Minimizable',
		type = 'bool',
		value = false,
		OnChange= function(self) MakeMinimapWindow() end,
		path = minimap_path,
		noHotkey = true,
	},
	lblblank1 = {name=' ', type='label'},
	leftClickOnMinimap = {
		name = 'Left Click Behaviour',
		type = 'radioButton',
		value = 'camera',
		items={
			{key='unitselection', name='Unit Selection'},
			{key='situational', name='Context Dependant'},
			{key='camera', name='Camera Movement'},
		},
		path = minimap_path,
		noHotkey = true,
	},	
	fadeMinimapOnZoomOut = {
		name = "Minimap fading when zoomed out",
		type = 'radioButton',
		value = 'none',
		items={
			{key='full', name='Full'},
			{key='partial', name='Semi-transparent'},
			{key='none', name='None'},
		},
		OnChange = function(self)
			last_alpha = 2 --invalidate last_alpha so it needs to be recomputed, for the background opacity
			end,
		path = minimap_path,
		noHotkey = true,
	},
	--[[
	simpleMinimapColors = {
		name = 'Simplified Minimap Colors',
		type = 'bool',
		desc = 'Show minimap blips as green for you, teal for allies and red for enemies (only minimap will use this simple color scheme).', 
		springsetting = 'SimpleMiniMapColors',
		OnChange = function(self) Spring.SendCommands{"minimap simplecolors " .. (self.value and 1 or 0) } end,
		path = minimap_path,
	},
	--]]
}

function WG.Minimap_SetOptions(aspect, opacity, resizable, buttonRight, minimizable)
	if aspect == 'arwindow' or aspect == 'armap' or aspect == 'arnone' then 
		options.use_map_ratio.value = aspect
	end
	options.opacity.value = opacity
	options.alwaysResizable.value = resizable
	options.buttonsOnRight.value = buttonRight
	options.minimizable.value = minimizable

	options.opacity.OnChange(options.opacity)
	options.use_map_ratio.OnChange(options.use_map_ratio)
	options.alwaysResizable.OnChange(options.alwaysResizable)
end

function updateRadarColors()
	local losViewOffBrightness = 0.5

	-- local fog = options.radar_fog_color.value
	-- local los = options.radar_los_color.value
	local fog_value = options.radar_fog_brightness1.value * losViewOffBrightness
	local los_value = (losViewOffBrightness - fog_value)
	local fog = {fog_value, fog_value, fog_value, 1}
	local los = {los_value, los_value, los_value, 1}
	local radar = options.radar_radar_color.value
	local jam = options.radar_jammer_color.value
	local radar2 = options.radar_radar2_color.value
	if usingNewEngine then
		Spring.SetLosViewColors(fog, los, radar, jam, radar2)
	else
		Spring.SetLosViewColors(
			{ fog[1], los[1], radar[1], jam[1]},
			{ fog[2], los[2], radar[2], jam[2]}, 
			{ fog[3], los[3], radar[3], jam[3]} 
		)
	end
end

local firstUpdate = true
local updateRunOnceRan = false

function widget:Update() --Note: these run-once codes is put here (instead of in Initialize) because we are waiting for epicMenu to initialize the "options" value first.
	if firstUpdate then
		firstUpdate = false
		return
	end
	local cs = Spring.GetCameraState()
	if not options.hideOnOverview.value then
		if cs.name == "ov" and not tabbedMode then
			Chili.Screen0:RemoveChild(window)
			tabbedMode = true
		end
		if cs.name ~= "ov" and tabbedMode then
			Chili.Screen0:AddChild(window)
			window:BringToFront()
			tabbedMode = false
		end
	end
	-- widgetHandler:RemoveCallIn("Update") -- remove update call-in since it only need to run once. ref: gui_ally_cursors.lua by jK
end

function widget:GameStart()
end

MakeMinimapWindow = function()
	if (window) then
		window:Dispose()
	end

	if options.disableMinimap.value then
		return
	end
	
	-- Set the size for the default settings.
	local screenWidth,screenHeight = Spring.GetWindowGeometry()
	local width, height = screenWidth/6, screenWidth/6
	
	if options.buttonsOnRight.value then
		width = width + iconsize
	else
		height = height + iconsize
	end
	
	if height > 0 and width > 0 and screenHeight > 0 and screenWidth > 0 then
		if width/height > screenWidth/screenHeight then
			screenHeight = height*screenWidth/width
		else
			screenWidth = width*screenHeight/height
		end
	end

	local map_panel_bottom = iconsize*1.3
	local map_panel_right = 0

	local buttons_height = iconsize+3
	local buttons_width = iconsize*13
	if options.buttonsOnRight.value then
		map_panel_bottom = 0
		map_panel_right = iconsize*1.3
		buttons_height = iconsize*13
		buttons_width = iconsize+3
	end

	map_panel = Chili.Panel:New {
		--classname = "bottomLeftPanel",
		x = 0,
		y = 0,
		bottom = map_panel_bottom,
		right = map_panel_right,

		margin = {0,0,0,0},
		padding = {8,8,8,8},
		backgroundColor = bgColor_panel,
	}


	window = Chili.Window:New{
		parent = Chili.Screen0,
		name   = 'Minimap Window',
		color = {0, 0, 0, 0},
		padding = {0, 0, 0, 0},
		width = (window and window.width) or width,
		height = (window and window.height) or height,
		x = (window and window.x) or 0,
		y = (window and window.y) or 0,
		-- dockable = true,
		draggable = false,
		resizable = options.alwaysResizable.value,
		minimizable = options.minimizable.value,
		tweakDraggable = true,
		tweakResizable = true,
		dragUseGrip = false,
		minWidth = 100,
		minHeight = 100,
		maxWidth = screenWidth,
		maxHeight = screenHeight,
		fixedRatio = options.use_map_ratio.value == 'arwindow',
	}
	window:BringToFront()

	options.use_map_ratio.OnChange(options.use_map_ratio)

	fakewindow = Chili.Panel:New{
		backgroundColor = {1,1,1, final_opacity},
		parent = window,
		x = 0,
		y = 0,
		width = "100%",
		height = "100%",
		dockable = false;
		draggable = false,
		resizable = false,
		padding = {0, 0, 0, 0},
		children = {
			map_panel,
			((not options.hidebuttons.value) and buttons_panel) or nil,
		},
	}
end

local leftClickDraggingCamera = false


 --// similar properties to "widget:Update(dt)" above but update less often.
-- function widget:KeyRelease(key, mods, label, unicode)
-- 	if key == 0x009 then --// "0x009" is equal to "tab". Reference: uikeys.txt

-- 	end
-- end

local function CleanUpFBO()
  if (gl.DeleteFBO) and fbo ~= nil then
    gl.DeleteFBO(fbo or 0)
    fbo = nil
  end
end

WG.uiScale = 1

function widget:Initialize()
	Spring.SendCommands("minimap geo " .. Spring.GetConfigString("MiniMapGeometry"))
	Spring.SendCommands("minimap simplecolors 1")
	Spring.SendCommands("minimap unitexp 0.3")
	Spring.SendCommands("minimap unitsize 1.2")
	if (Spring.GetMiniMapDualScreen()) then
		Spring.Echo("ChiliMinimap: auto disabled (DualScreen is enabled).")
		widgetHandler:RemoveWidget()
		return
	end

	if (not WG.Chili) then
		widgetHandler:RemoveWidget()
		return
	end

	Chili = WG.Chili

	MakeMinimapWindow()

	if (gl.CreateFBO) then
		fbo = gl.CreateFBO()

		fbo.color0 = nil

		gl.DeleteTextureFBO(offscreentex or 0)

		local vsx,vsy = gl.GetViewSizes()
		if vsx > 0 and vsy > 0 then
		  offscreentex = gl.CreateTexture(vsx,vsy, {
		    border = false,
		    min_filter = GL.LINEAR,
		    mag_filter = GL.LINEAR,
		    wrap_s = GL.CLAMP,
		    wrap_t = GL.CLAMP,
		    fbo = true,
		  })

		  fbo.color0 = offscreentex
		  fbo.drawbuffers = GL_COLOR_ATTACHMENT0_EXT
		end

		if (gl.CreateShader) then
			fadeShader = gl.CreateShader({
			vertex = [[
				varying vec2 texCoord;

          void main() {
			texCoord = gl_Vertex.xy * 0.5 + 0.5;
			texCoord /= 1.3;
			texCoord.y += 0.17;
			texCoord.x += 0.04;
			gl_Position = vec4(gl_Vertex.xyz, 1.0);
          }
			]],
		    fragment = [[
		      uniform sampler2D tex0;
		      uniform float alpha;
		      uniform vec4 bounds;
		      uniform vec2 screen;

		      varying vec2 texCoord;

		      const float edgeFadePixels = 16.0;

		      void main(void) {
		        vec4 color = texture2D(tex0, texCoord.st);
		        //float width = bounds.z;
		        //float height = bounds.w;
		        float edgeFadeScaledPixels = edgeFadePixels/1080.0 * screen.y * 20;
		        vec2 edgeFadeBase = vec2(edgeFadeScaledPixels / screen.x, edgeFadeScaledPixels / screen.y);
				vec2 edgeFade = vec2((2.0 * bounds.z) / edgeFadeBase.x, (2.0 * bounds.w) / edgeFadeBase.y);
				//bounds.y -= 0.015;
				//bounds.x += 0.005;
				vec2 edgeAlpha = vec2(clamp(1.0 - abs((texCoord.x - bounds.x)/bounds.z - 0.5) * 2, 0.0, 1.0/edgeFade.x) * edgeFade.x,
										clamp(1.0 - abs((texCoord.y - bounds.y)/bounds.w - 0.5) * 2.0, 0.0, 1.0/edgeFade.y) * edgeFade.y);
				float final_alpha = edgeAlpha.x * edgeAlpha.y * alpha;
				color.r = color.r;
				gl_FragColor = vec4(color.rgb, final_alpha);
		      }
		    ]],
		    uniformInt = {
		      tex0 = 0,
		    },
		    uniform = {
              alpha = 0,
              bounds = {0,0,0,0},
              screen = {0,0},
			},
		  })

		  if (fadeShader == nil) then
		    Spring.Log(widget:GetInfo().name, LOG.ERROR, "Minimap widget: fade shader error: "..gl.GetShaderLog())
			  CleanUpFBO()
		  else
			  alphaLoc = gl.GetUniformLocation(fadeShader, 'alpha')
				boundsLoc = gl.GetUniformLocation(fadeShader, 'bounds')
				screenLoc = gl.GetUniformLocation(fadeShader, 'screen')
		  end
		else --Shader Generation impossible, clean up FBO
		  CleanUpFBO()
		end
	end

	gl.SlaveMiniMap(true)
end

function widget:Shutdown()
	--// reset engine default minimap rendering
	gl.SlaveMiniMap(false)

  if (gl.DeleteTextureFBO) then
    gl.DeleteTextureFBO(offscreentex)
  end

  CleanUpFBO()

	--// free the chili window
	if (window) then
		window:Dispose()
	end

	WG.MinimapDraggingCamera = nil
end

local lx, ly, lw, lh, last_window_x, last_window_y

local function DrawMiniMap()
  gl.Clear(GL.COLOR_BUFFER_BIT,0,0,0,0)
  glDrawMiniMap()
end

WG.uiScale = 1.3

function widget:DrawScreen()
	local cs = Spring.GetCameraState()
	if (options.disableMinimap.value or window.hidden or cs.name == "ov") then 
		gl.ConfigMiniMap(0,0,0,0) --// a phantom map still clickable if this is not present.
		lx = 0
		ly = 0
		lh = 0
		lw = 0
		return
	end

	local cx,cy,cw,ch = Chili.unpack4(map_panel.clientArea)

	if (options.use_map_ratio.value == 'armap') then
		cx,cy,cw,ch = AdjustMapAspectRatioToWindow(cx,cy,cw,ch)
	end

	local vsx,vsy = gl.GetViewSizes()
	if (lw ~= cw or lh ~= ch or lx ~= cx or ly ~= cy or last_window_x ~= window.x or last_window_y ~= window.y) then
		lx = cx
		ly = cy
		lh = ch
		lw = cw
		last_window_x = window.x
		last_window_y = window.y

		cx,cy = map_panel:LocalToScreen(cx,cy)
		gl.ConfigMiniMap(cx*(WG.uiScale or 1),(vsy-ch-cy)*(WG.uiScale or 1),cw*(WG.uiScale or 1),ch*(WG.uiScale or 1))
	end

	-- Do this even if the fadeShader can't exist, just so that all hiding code still behaves properly
	local alpha = 1
	local alphaMin = options.fadeMinimapOnZoomOut.value == 'full' and 0.0 or 0.3
	if options.fadeMinimapOnZoomOut.value ~= 'none' then
		if WG.COFC_SkyBufferProportion ~= nil then --if nil, COFC is not enabled
			alpha = 1 - (WG.COFC_SkyBufferProportion)
		else
			local height = cs.py
			if cs.height ~= null then height = cs.height end
			--NB: Value based on engine 98.0.1-403 source for OverheadController's maxHeight member variable calculation.
			local maxHeight = 9.5 * math.max(Game.mapSizeX, Game.mapSizeZ)/Game.squareSize
			if options.fadeMinimapOnZoomOut.value == 'full' then
				alpha = 1 - ((height - (maxHeight * 0.7)) / (maxHeight * 0.3)) -- 0 to 1
			else
				alpha = 1 - (height - (maxHeight * 0.7)) -- 0.3 to 1
			end
		end

		--Guarantees a buffer of 0 alpha near full zoom-out, to help account for the camera following the map's elevation
		if alpha < 1 then alpha = math.min(math.max((alpha - 0.2) / 0.8, alphaMin), 1.0) end 
	end

	if math.abs(last_alpha - alpha) > 0.0001 then
		final_opacity = options.fadeMinimapOnZoomOut.value == 'full' and options.opacity.value * alpha or options.opacity.value
		last_alpha = alpha

		fakewindow.backgroundColor = {1,1,1, final_opacity}
		local final_map_bg_color = options.fadeMinimapOnZoomOut.value == 'full' and bgColor_panel[4]*alpha or bgColor_panel[4]
		map_panel.backgroundColor = {1,1,1, final_map_bg_color}
		if alpha < 0.1 then 
			fakewindow.children = {map_panel} 
		else 
			fakewindow.children = {map_panel, buttons_panel} 
		end 
		fakewindow:Invalidate()
		map_panel:Invalidate()
	end

	gl.PushAttrib(GL.ALL_ATTRIB_BITS)
	gl.MatrixMode(GL.PROJECTION)
	gl.PushMatrix()
	gl.MatrixMode(GL.MODELVIEW)
	gl.PushMatrix()

	if fbo ~= nil and fadeShader ~= nil then
		gl.ActiveFBO(fbo, DrawMiniMap)
		gl.Blending(true)

		-- gl.Color(1,1,1,alpha)
		gl.Texture(0, offscreentex)
		gl.UseShader(fadeShader)
		gl.Uniform(alphaLoc, alpha * 7)
		local px, py = window.x + lx * 2.5, vsy - window.y - ly * 3
		gl.Uniform(boundsLoc, (px/vsx) * 7, ((py - lh)/vsy) * 1.03, (lw/vsx) / 1.8, (lh/vsy) * 0.60)
		gl.Uniform(screenLoc, vsx, vsy)
		-- Spring.Echo("Bounds: "..(window.x + lx)/vsx..", "..(window.y + ly)/vsy..", "..((window.x + lx) + lw)/vsx..", "..((window.y + ly) + lh)/vsy)
		gl.TexRect(-1, 1, 1, -1)

		gl.Texture(0, false)
		gl.Blending(false)
		gl.UseShader(0)
	elseif (alpha > 0.01) then
		glDrawMiniMap()
	end

	gl.MatrixMode(GL.PROJECTION)
	gl.PopMatrix()
	gl.MatrixMode(GL.MODELVIEW)
	gl.PopMatrix()
	gl.PopAttrib()
end


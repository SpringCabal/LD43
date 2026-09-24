function widget:GetInfo()
	return {
		name      = "Minimap Viewport",
		desc      = "Configurable minimap which can view a limited section of the map",
		author    = "GoogleFrog", -- based on chili minimap shader and viewport by gajop.
		date      = "5 December 2018",
		license   = "GNU GPL, v2 or later",
		layer     = -100000,
		enabled   = true,
	}
end

---------------------------------------------------------------
---------------------------------------------------------------
-- Configuration

local EDGE_FADE_PIXELS = 16 -- Fade on minimap edge.
local MINIMAP_SCALE = 1.3 -- Scale of drawn minimap. Only works in the range (0, 2).
local SOURCE_SIZE = 600 -- Has to be at least half window height? Unsure
local SCREEN_X = 0 -- Position on screen. Range of [0, 1].
local SCREEN_Y = 0 -- Position on screen. Range of [0, 1].

local MINIMAP_ALPHA = 1

-- Width and height of the player-centered minimap viewport in world units.
local VIEWPORT_WORLD_X = 8000 - 3300
local VIEWPORT_WORLD_Z = 8100 - 3400

local function GetPlayerViewport()
	local bloodmageDefID = UnitDefNames["bloodmage"].id
	local units = Spring.GetTeamUnits(Spring.GetMyTeamID())
	for i = 1, #units do
		if Spring.GetUnitDefID(units[i]) == bloodmageDefID then
			local x, _, z = Spring.GetUnitPosition(units[i])
			if x then
				-- Keep the same fixed-size crop centered on the bloodmage, including
				-- when the crop extends beyond a map edge. The minimap texture is
				-- clamped at the boundary, so its layout and zoom never change.
				return x - VIEWPORT_WORLD_X * 0.5,
					z - VIEWPORT_WORLD_Z * 0.5,
					VIEWPORT_WORLD_X, VIEWPORT_WORLD_Z, MINIMAP_SCALE
			end
		end
	end
	local fallbackX = (Game.mapSizeX - VIEWPORT_WORLD_X) * 0.5
	local fallbackZ = (Game.mapSizeZ - VIEWPORT_WORLD_Z) * 0.5
	return fallbackX, fallbackZ, VIEWPORT_WORLD_X, VIEWPORT_WORLD_Z, MINIMAP_SCALE
end

---------------------------------------------------------------
---------------------------------------------------------------
-- Constants and Stuff

local GL_COLOR_ATTACHMENT0_EXT = 0x8CE0

-- GLSL will not initialise a float from an int literal, and tostring() drops the
-- fractional part of whole numbers, so spell the decimal point out explicitly.
local function GlslFloat(x)
	local str = tostring(x)
	if not str:find("[.eE]") then
		str = str .. ".0"
	end
	return str
end

local glDrawMiniMap = gl.DrawMiniMap
local glResetState = gl.ResetState
local glResetMatrices = gl.ResetMatrices

---------------------------------------------------------------
---------------------------------------------------------------
-- GL Variables

local fbo
local offscreentex

local fadeShader
local alphaLoc
local sourceBoundsLoc
local screenRectLoc

---------------------------------------------------------------
---------------------------------------------------------------

local function CleanUpFBO()
	if (gl.DeleteFBO) and fbo ~= nil then
		gl.DeleteFBO(fbo or 0)
		fbo = nil
	end
end

local function DrawMiniMap()
	gl.Clear(GL.COLOR_BUFFER_BIT,0,0,0,0)
	glDrawMiniMap()
end

function widget:Initialize()
	Spring.SendCommands("minimap geo " .. Spring.GetConfigString("MiniMapGeometry"))
	Spring.SendCommands("minimap simplecolors 1")
	Spring.SendCommands("minimap unitexp 0.3")
	Spring.SendCommands("minimap unitsize 1.2")
	Spring.SendCommands("minimap fullproxy 0")

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
					varying vec2 mapCoord;

					void main() {
						mapCoord = gl_Vertex.xy * 0.5 + 0.5;
						gl_Position = vec4(gl_Vertex.xyz, 1.0);
					}
				]],
				fragment = [[
					uniform sampler2D tex0;
					uniform float alpha;
					uniform vec4 sourceBounds;
					uniform vec4 screenRect;
					uniform vec2 screen;

					varying vec2 mapCoord;

					const float edgeFadePixels = ]] .. GlslFloat(EDGE_FADE_PIXELS) .. [[;

					void main(void) {
						vec2 screenPosition = vec2(
							mapCoord.x * screen.x,
							(1.0 - mapCoord.y) * screen.y);
						vec2 cropCoord = (screenPosition - screenRect.xy) / screenRect.zw;
						if (cropCoord.x < 0.0 || cropCoord.x > 1.0 ||
							cropCoord.y < 0.0 || cropCoord.y > 1.0) {
							discard;
						}
						vec2 texCoord = vec2(
							sourceBounds.x + cropCoord.x * sourceBounds.z,
							sourceBounds.y + (1.0 - cropCoord.y) * sourceBounds.w);
						vec4 color = texture2D(tex0, texCoord.st);
						float edgeFadeScaledPixels = edgeFadePixels/1080.0 * screen.y * 20.0;
						vec2 edgeFade = clamp(edgeFadeScaledPixels / screenRect.zw, 0.0, 1.0);
						vec2 edgeAlpha = min(cropCoord, 1.0 - cropCoord) / max(edgeFade, vec2(0.0001));
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
					sourceBounds = {0,0,0,0},
					screenRect = {0,0,0,0},
					screen = {0,0},
				},
			})

			if (fadeShader == nil) then
				Spring.Log(widget:GetInfo().name, LOG.ERROR, "Minimap widget: fade shader error: "..gl.GetShaderLog())
				CleanUpFBO()
			else
				alphaLoc = gl.GetUniformLocation(fadeShader, 'alpha')
				sourceBoundsLoc = gl.GetUniformLocation(fadeShader, 'sourceBounds')
				screenRectLoc = gl.GetUniformLocation(fadeShader, 'screenRect')
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
end

local needConfig = true
function widget:DrawScreen()
	local vsx,vsy = gl.GetViewSizes()

	if needConfig then
		gl.ConfigMiniMap(0, vsy/2, SOURCE_SIZE, SOURCE_SIZE)
		needConfig = false
	end
	local mapXWorld, mapZWorld, viewportWorldX, viewportWorldZ, minimapScale = GetPlayerViewport()
	local minimapPosX, minimapPosY, minimapSizeX, minimapSizeY = Spring.GetMiniMapGeometry()
	local sourceLeft = (minimapPosX + minimapSizeX * mapXWorld / Game.mapSizeX) / vsx
	local sourceBottom = (minimapPosY + minimapSizeY * (1 - (mapZWorld + viewportWorldZ) / Game.mapSizeZ)) / vsy
	local sourceWidthCrop = minimapSizeX * viewportWorldX / Game.mapSizeX / vsx
	local sourceHeightCrop = minimapSizeY * viewportWorldZ / Game.mapSizeZ / vsy
	local screenWidth = minimapSizeX * viewportWorldX / Game.mapSizeX * minimapScale
	local screenHeight = minimapSizeY * viewportWorldZ / Game.mapSizeZ * minimapScale

	gl.PushAttrib(GL.ALL_ATTRIB_BITS)
	gl.MatrixMode(GL.PROJECTION)
	gl.PushMatrix()
	gl.MatrixMode(GL.MODELVIEW)
	gl.PushMatrix()

	if fbo ~= nil and fadeShader ~= nil then
		gl.ActiveFBO(fbo, DrawMiniMap)
		gl.Blending(true)

		gl.Texture(0, offscreentex)
		gl.UseShader(fadeShader)
		gl.Uniform(alphaLoc, MINIMAP_ALPHA * 3)
		gl.Uniform(sourceBoundsLoc, sourceLeft, sourceBottom, sourceWidthCrop, sourceHeightCrop)
		gl.Uniform(screenRectLoc, SCREEN_X * vsx, SCREEN_Y * vsy, screenWidth, screenHeight)

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

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

-- Top left, width and height of minimap as a proportion of map size.
local MAP_X = 3300 / Game.mapSizeX
local MAP_Z = 3400 / Game.mapSizeZ
local MAP_X_SIZE = 8000 / Game.mapSizeX - MAP_X
local MAP_Z_SIZE = 8100 / Game.mapSizeZ - MAP_Z

---------------------------------------------------------------
---------------------------------------------------------------
-- Constants and Stuff

local GL_COLOR_ATTACHMENT0_EXT = 0x8CE0

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
local boundsLoc
local mainBoundsLoc

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
					varying vec2 texCoord;
					uniform vec2 mainbounds;

					void main() {
						texCoord = gl_Vertex.xy * 0.5 + 0.5;
						texCoord /= ]] .. MINIMAP_SCALE ..[[;
						texCoord.y += mainbounds.y;
						texCoord.x += mainbounds.x;
						gl_Position = vec4(gl_Vertex.xyz, 1.0);
					}
				]],
				fragment = [[
					uniform sampler2D tex0;
					uniform float alpha;
					uniform vec4 bounds;
					uniform vec2 screen;

					varying vec2 texCoord;

					const float edgeFadePixels = ]] .. EDGE_FADE_PIXELS .. [[;

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
					mainbounds = {0,0},
					screen = {0,0},
				},
			})

			if (fadeShader == nil) then
				Spring.Log(widget:GetInfo().name, LOG.ERROR, "Minimap widget: fade shader error: "..gl.GetShaderLog())
				CleanUpFBO()
			else
				alphaLoc = gl.GetUniformLocation(fadeShader, 'alpha')
				boundsLoc = gl.GetUniformLocation(fadeShader, 'bounds')
				mainBoundsLoc = gl.GetUniformLocation(fadeShader, 'mainbounds')
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
		
		gl.Uniform(boundsLoc,
			MAP_X * SOURCE_SIZE/vsx, 
			1 - (MAP_Z + MAP_Z_SIZE)*SOURCE_SIZE/vsy,
			MAP_X_SIZE*SOURCE_SIZE/vsx, 
			MAP_Z_SIZE*SOURCE_SIZE/vsy
		)
		
		gl.Uniform(mainBoundsLoc,
			(MAP_X*SOURCE_SIZE/vsx) - SCREEN_X/MINIMAP_SCALE, 
			1 - 1/MINIMAP_SCALE + SCREEN_Y/MINIMAP_SCALE - ((MAP_Z)*SOURCE_SIZE/vsy)
		)
		
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


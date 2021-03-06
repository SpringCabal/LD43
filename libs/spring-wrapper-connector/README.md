# spring-wrapper-connector
Handles Lua communication with the local wrapper program

# Features
- Simple JSON based communication protocol with spring-launcher

## Dependencies
- [spring-launcher](https://github.com/gajop/spring-launcher/)
- [json.lua](https://github.com/Spring-SpringBoard/SpringBoard-Core/blob/master/libs_sb/json.lua)

## Install
1. Obtain the repository either by adding it as a git submodule or by copying the entire structure in to your Spring game folder. Put it anywhere (although `/libs` is suggested and used by default).
2. Copy the file `api_connector_loader.lua` to the `luaui/widgets folder` and modify the SPRING_WRAPPER_CONNECTOR_DIR path as necessary.

## Usage
```lua

-- Send command
WG.Connector.Send(commandName, data)
-- Example:
WG.Connector.Send("OpenFile", {
    path = "/home/myuser/some_file"
})

-- Register command handler
WG.Connector.Register(commandName, callback)
-- Example:
WG.Connector.Register("FileOpened", function(command)
    Spring.Echo("File opened: " .. tostring(command.path))
end)
```
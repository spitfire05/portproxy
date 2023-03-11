PLUGINS = {}

local bindump = require("plugins/bindump")
LOG_INFO = {}
LOG_WARN = {}
LOG_ERROR = {}
table.insert(PLUGINS, bindump)

BUILD_LOGGER = function(name)
    return {
        info = function(msg)
            LOG_INFO(name, msg)
        end,
        warn = function(msg)
            LOG_WARN(name, msg)
        end,
        error = function(msg)
            LOG_ERROR(name, msg)
        end
    }
end

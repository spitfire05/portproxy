return {
    name = "bindump",
    on_rx = function(data)
        local file = io.open("bindump", "ab")
        local log = BUILD_LOGGER("bindump")
        if (file ~= nil) then
            local size = 0
            io.output(file)
            for i, d in ipairs(data) do
                io.write(string.char(d))
                size = size + 1;
            end
            log.info(string.format("Written %d bytes", size))
            io.flush()
        else
            log.error("Cannot open output file")
        end
    end
}

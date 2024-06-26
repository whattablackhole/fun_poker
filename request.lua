local http = require("resty.http")

local _M = {}

function _M.fetch_game_server(lobby_id)
    local httpc = http.new()
    
    local res, err = httpc:request_uri("http://127.0.0.1:8081/assign_server", {
        method = "GET",
        query = {
            lobby_id = lobby_id
        }
    })

    if res and res.status == 200 then
        return res.body:gsub("%s+", "")
    else
        ngx.log(ngx.ERR, "Failed to fetch game server: ", err)
        return nil, err
    end
end

return _M
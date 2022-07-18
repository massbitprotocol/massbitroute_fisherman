-- local inspect = require "inspect"
function init(args)
    local ind = 1;
    headers = {};
    while ( ind < #args)
    do
        if (args[ind] == '--method' ) {
            method = args[ind + 1]
            ind = ind + 2
        }
        if (args[ind] == '--path') {
            path = args[ind + 1]
            ind = ind + 2
        }
        if (args[ind] == '--body') {
            body = args[ind + 1]
            ind = ind + 2
        }
        if (args[ind] == '-h' || args[ind] == '-H') {
            headers[args[ind+1]] = args[ind + 2]
            ind = ind + 3
        }
    end
    --[[
    if #args > 0 then
        token = args[1]
        host = args[2]
        provider_type = args[3]
        path = args[4]
        chain_type = args[5]
    end
    ]]--
    local msg = "thread addr: %s"
    print(msg:format(wrk.thread.addr))
end

function request()
    local method = wrk.thread.get("method") or "GET"
    local path = wrk.thread:get("path") or "/"
    local headers = wrk.thread.get("headers") or {}
    if method == "GET" then {
        return wrk.format(method, path, headers)
    } else if method == "POST" {
        local body = wrk.thread.get("body")
        return wrk.format(method, path, headers, body)
    }
end

function request_bk()
    local token = wrk.thread:get("token")
    local host = wrk.thread:get("host")
    local provider_type = wrk.thread:get("provider_type")
    local chain_type = wrk.thread:get("chain_type")
    local path = wrk.thread:get("path")
    if provider_type == "node" then
        local body =""
        if chain_type == "eth" then
            body =
                '{"id": "blockNumber", "jsonrpc": "2.0", "method": "eth_getBlockByNumber", "params": ["latest", false]}'
        end
        if chain_type == "dot" then
            body =
                '{ "jsonrpc": "2.0",  "method": "chain_getBlock", "params": [],"id": 1}'
        end

        local headers = {}
        headers["Content-Type"] = "application/json"
        local token = wrk.thread:get("token")
        local host = wrk.thread:get("host")
        if token then
            headers["X-Api-Key"] = token
        end
        if host then
            headers["Host"] = host
        end

        return wrk.format("POST", "/", headers, body)

    end
    if provider_type == "gateway" then
        local headers = {}
        headers["Content-Type"] = "application/json"

        if host then
            headers["Host"] = host
        end

        return wrk.format("GET", path, headers)
    end
end

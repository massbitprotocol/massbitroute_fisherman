-- local inspect = require "inspect"
function init(args)
    local ind = 1;
    while ( ind < #args)
    do
        if (args[ind] == '--method' ) then
            method = args[ind + 1];
            ind = ind + 2;
        end
        if (args[ind] == '--body') then
            body = args[ind + 1];
            ind = ind + 2;
        end
    end
    local msg = "thread addr: %s"
    print(msg:format(wrk.thread.addr))
end

function request()
    if method == "GET" then
        return wrk.format(method, wrk.path)
    elseif method == "POST" then
        local body = wrk.thread:get("body")
        return wrk.format(method, wrk.path, nil, body)
    end
end


function response(status, headers, body)
    --print(status)
    --print(body)
end

function done(summary, latency, requests)

end
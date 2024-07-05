FROM openresty/openresty:1.21.4.1-0-alpine-fat

RUN /usr/local/openresty/luajit/bin/luarocks install lua-resty-redis
RUN /usr/local/openresty/luajit/bin/luarocks install lua-resty-jwt
RUN /usr/local/openresty/luajit/bin/luarocks install lua-cjson
RUN /usr/local/openresty/luajit/bin/luarocks install lua-resty-http

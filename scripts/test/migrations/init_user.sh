#!/bin/bash
psql -U postgres -d massbit-user -c "insert into users (id,username,wallet_address,email,password,provider,avatar_url,verify_token,salt,roles,status,created_at,updated_at) values ('b1cf032d-5971-410a-90fe-9c454a4173f0','demo','5CVbMk8jB6Xo96FMxCgzXVSP8wSrmVH5R1FP1UY41JSTQKwo','hoang@codelight.co','\$2b\$12\$ic59AzyivGVu/pGqhmRRIep4fqjKmON7c65bQxph8fcs40JEG2s4.','local',NULL,'WRMXH','\$2b\$10\$KQO1wQ0NkxGp7tRjFnkIWe','{user}','active','2022-07-27 02:49:29.972127','2022-07-29 05:07:52.655309')"
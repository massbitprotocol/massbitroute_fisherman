sudo apt update -y
sudo apt install build-essential pkg-config libssl-dev supervisor curl nginx -y
sudo apt install software-properties-common
sudo mkdir /opt/fishman
sudo cp /home/viettai/scheduler /opt/fishman/
#-------------------------------------------
# Setup scheduler
#-------------------------------------------

echo '#!/bin/bash

export RUST_LOG=info
export RUST_LOG_TYPE=console
export DATABASE_URL=postgres://fisherman:FishermanCodelight123@35.193.163.173:5432/massbit-fisherman
export DOMAIN=massbitroute.net
export URL_GATEWAYS_LIST=https://portal.massbitroute.net/mbr/gateway/list/verify
export URL_NODES_LIST=https://portal.massbitroute.net/mbr/node/list/verify

/opt/fishman/scheduler
' | sudo tee /opt/fishman/run.sh

#-------------------------------------------
#  Set up supervisor
#-------------------------------------------
echo '[program:scheduler]
command=bash /opt/fishman/run.sh
autostart=true
autorestart=true
stderr_logfile=/var/log/scheduler.err.log
stdout_logfile=/var/log/scheduler.out.log
user=root
stopasgroup=true' | sudo tee /etc/supervisor/conf.d/scheduler.conf

echo 'server {
    server_name scheduler.fishman.massbitroute.net;
    listen 443 ssl;
    ssl_certificate /opt/ssl/live/fishman.massbitroute.net/fullchain.pem;
    ssl_certificate_key /opt/ssl/live/fishman.massbitroute.net/privkey.pem;
    location / {
      #try_files $uri $uri/ =404;
      #proxy_buffering off;
      proxy_pass http://127.0.0.1:3031;
      #proxy_set_header X-Real-IP $remote_addr;
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
      #proxy_http_version 1.1;
      #proxy_set_header Upgrade $http_upgrade;
      #proxy_set_header Connection "upgrade";
      #proxy_read_timeout 86400s;
      #proxy_send_timeout 86400s;
    }
}' | sudo tee /etc/nginx/sites-available/scheduler

sudo ln -sf /etc/nginx/sites-available/scheduler /etc/nginx/sites-enabled/scheduler
sudo service nginx restart

sudo chmod 770 /opt/fisherman/run.sh
sudo chmod 770 /opt/fisherman/scheduler
sudo supervisorctl reread
sudo supervisorctl update
sudo supervisorctl restart scheduler

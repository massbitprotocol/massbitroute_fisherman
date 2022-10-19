sudo rm /opt/fisherman -rf

sudo cp ~/fisherman /opt/ -r
sudo mv /opt/fisherman/fisherman.conf /etc/supervisor/conf.d/fisherman.conf
sudo chmod +x /opt/fisherman/run.sh
sudo mv /opt/fisherman/configs.production /opt/fisherman/configs

sudo rm -rf ~/fisherman

sudo supervisorctl restart fisherman

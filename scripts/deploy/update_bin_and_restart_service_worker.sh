sudo rm /opt/fisherman -rf

sudo cp ~/fisherman /opt/ -r
sudo chmod +x /opt/fisherman/run.sh
sudo rm -rf ~/fisherman

sudo supervisorctl restart fisherman

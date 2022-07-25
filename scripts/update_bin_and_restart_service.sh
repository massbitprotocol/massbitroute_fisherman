sudo mv ~/scheduler /opt/fisherman/
sudo mv ~/run.sh /opt/fisherman/
chmod +x /opt/fisherman/run.sh

sudo cp ~/configs /opt/fisherman/ -r
sudo rm -rf ~/configs

sudo supervisorctl restart scheduler

sudo mv ~/scheduler /opt/fisherman/
sudo mv ~/run.sh /opt/fisherman/

sudo cp ~/configs /opt/fisherman/ -r
sudo rm -rf ~/configs

sudo supervisorctl restart scheduler

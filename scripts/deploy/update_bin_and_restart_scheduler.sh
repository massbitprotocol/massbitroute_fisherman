sudo rm /opt/fisherman -rf
sudo mkdir /opt/fisherman
sudo mkdir /opt/fisherman/configs


sudo mv ~/scheduler /opt/fisherman/
sudo mv ~/.env /opt/fisherman/
sudo mv ~/run.sh /opt/fisherman/
sudo mv ~/common.json /opt/fisherman/configs/
sudo mv ~/log.yaml /opt/fisherman/configs/

chmod +x /opt/fisherman/run.sh

#sudo cp ~/configs /opt/fisherman/ -r
#sudo rm -rf ~/configs

sudo supervisorctl restart scheduler

sudo chown -R root:root . && sudo chmod -R 777 .
sudo chmod +x /root && sudo chown -R root:root /root && sudo chmod -R 777 /root
setup:
	cd scripts && sudo chmod +x redeploy.sh
	./setup.sh
redeploy:
	cd scripts && sudo chmod +x redeploy.sh
	./redeploy.sh
renew:
	cd scripts && sudo chmod +x redeploy.sh
	./renew.sh
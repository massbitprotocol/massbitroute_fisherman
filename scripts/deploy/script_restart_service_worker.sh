cd fisherman

tmux kill-session -t fisherman_worker
sleep 3
tmux new-session -d -s fisherman_worker 'bash run.sh >> fisherman.log 2>&1'

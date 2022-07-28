cd fisherman
SESSION=fisherman_worker
tmux kill-session -t $SESSION
sleep 3
tmux new-session -d -s $SESSION 'bash run.sh > fisherman.log 2>&1'

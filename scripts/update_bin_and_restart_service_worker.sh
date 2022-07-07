cd fisherman
SESSION=fisherman_worker
tmux kill-session -t $SESSION
tmux new-session -d -s $SESSION 'bash run.sh'

#!/bin/bash
strip ../../target/release/scheduler
scp ../../target/release/scheduler scheduler:~/
ssh -t scheduler "sudo docker cp /home/huy/scheduler mbr_fisherman_scheduler_95:/usr/local/bin/scheduler/"
ssh -t scheduler "sudo docker exec mbr_fisherman_scheduler_95 supervisorctl restart scheduler"

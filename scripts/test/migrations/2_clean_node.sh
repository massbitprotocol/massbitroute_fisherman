#!/bin/bash
psql -U postgres -d massbit-user -c "delete from mbr_nodes"
psql -U postgres -d massbit-user -c "delete from mbr_gateways"
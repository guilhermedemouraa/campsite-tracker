#!/bin/bash
source .env
aws ec2 stop-instances --instance-ids $INSTANCE_ID
echo "Instance stopped. Run start.sh to restart it."
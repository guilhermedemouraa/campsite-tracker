#!/bin/bash
source .env
aws ec2 start-instances --instance-ids $INSTANCE_ID
echo "Instance starting... Wait 2 minutes then run ./build_and_deploy.sh"
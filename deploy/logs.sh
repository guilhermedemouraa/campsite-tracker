#!/bin/bash
source .env

INSTANCE_IP=$(aws ec2 describe-instances \
    --instance-ids $INSTANCE_ID \
    --query 'Reservations[0].Instances[0].PublicIpAddress' \
    --output text)

ssh -i campsite-key.pem ec2-user@$INSTANCE_IP 'sudo docker logs -f campsite-tracker'
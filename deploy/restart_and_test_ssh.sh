#!/bin/bash
set -e
source .env
echo "🔄 Restarting EC2 instance..."
aws ec2 reboot-instances --instance-ids $INSTANCE_ID

echo "⏳ Waiting for instance to restart (this takes about 2-3 minutes)..."
sleep 30

# Wait for instance to be running
aws ec2 wait instance-running --instance-ids $INSTANCE_ID
echo "✅ Instance is running"

# Wait a bit more for SSH to be ready
echo "⏳ Waiting for SSH service to be ready..."
sleep 60

# Get the IP again (might change after reboot)
INSTANCE_IP=$(aws ec2 describe-instances \
    --instance-ids $INSTANCE_ID \
    --query 'Reservations[0].Instances[0].PublicIpAddress' \
    --output text)

echo "Instance IP after restart: $INSTANCE_IP"

# Test SSH connectivity
echo "🔍 Testing SSH connectivity..."
for i in {1..5}; do
    echo "Attempt $i/5..."
    if ssh -i campsite-key.pem -o StrictHostKeyChecking=no -o ConnectTimeout=10 ec2-user@$INSTANCE_IP 'echo "SSH works!"' 2>/dev/null; then
        echo "✅ SSH is working!"
        break
    else
        echo "❌ SSH not ready yet, waiting 30 seconds..."
        sleep 30
    fi
done
#!/bin/bash
set -e

# Load environment
source .env

echo "🔨 Building Docker image..."

# Go to project root (where Dockerfile is located)
cd ..

# Build the image locally (this handles cross-compilation automatically)
docker build -t campsite-tracker .

# Save image to tar file
docker save campsite-tracker > deploy/campsite-tracker.tar

# Go back to deploy directory
cd deploy

echo "📦 Deploying to AWS..."

# Get instance IP
INSTANCE_IP=$(aws ec2 describe-instances \
    --instance-ids $INSTANCE_ID \
    --query 'Reservations[0].Instances[0].PublicIpAddress' \
    --output text)

echo "Instance IP: $INSTANCE_IP"

# Copy image to EC2
echo "📤 Uploading Docker image..."
scp -i campsite-key.pem -o StrictHostKeyChecking=no \
    campsite-tracker.tar ec2-user@$INSTANCE_IP:/home/ec2-user/

# Load and run on EC2
echo "🚀 Starting container..."
ssh -i campsite-key.pem -o StrictHostKeyChecking=no ec2-user@$INSTANCE_IP << 'EOF'
    # Stop any existing container
    sudo docker stop campsite-tracker 2>/dev/null || true
    sudo docker rm campsite-tracker 2>/dev/null || true
    
    # Load new image
    sudo docker load < campsite-tracker.tar
    
    # Run container
    sudo docker run -d \
        --name campsite-tracker \
        -p 8080:8080 \
        --restart unless-stopped \
        campsite-tracker
        
    echo "Container started!"
    sudo docker ps
EOF

# Cleanup
rm campsite-tracker.tar

echo "✅ Deployment complete!"
echo "🌐 Your app is available at: http://$INSTANCE_IP:8080"
echo "📋 Check logs with: ssh -i campsite-key.pem ec2-user@$INSTANCE_IP 'sudo docker logs campsite-tracker'"
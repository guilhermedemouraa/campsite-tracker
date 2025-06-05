#!/bin/bash
set -e

# Load environment
source .env

echo "🔨 Building Docker image..."

# Go to project root (where Dockerfile is located)
cd ..

# Build the image locally (this handles cross-compilation automatically)
docker build -t campsite-tracker .

# Save and compress image to tar file
echo "📦 Compressing Docker image..."
docker save campsite-tracker | gzip > deploy/campsite-tracker.tar.gz

# Go back to deploy directory
cd deploy

echo "📦 Deploying to AWS..."

# Get instance IP
INSTANCE_IP=$(aws ec2 describe-instances \
    --instance-ids $INSTANCE_ID \
    --query 'Reservations[0].Instances[0].PublicIpAddress' \
    --output text)

echo "Instance IP: $INSTANCE_IP"

# Copy compressed image to EC2 using rsync for better reliability
echo "📤 Uploading compressed Docker image..."
rsync -avz --progress -e "ssh -i campsite-key.pem -o StrictHostKeyChecking=no" \
    campsite-tracker.tar.gz ec2-user@$INSTANCE_IP:/home/ec2-user/

# Load and run on EC2
echo "🚀 Starting container with database connection..."
ssh -i campsite-key.pem -o StrictHostKeyChecking=no ec2-user@$INSTANCE_IP << EOF
    # Stop any existing container
    sudo docker stop campsite-tracker 2>/dev/null || true
    sudo docker rm campsite-tracker 2>/dev/null || true
    
    # Load compressed image
    zcat campsite-tracker.tar.gz | sudo docker load
    
    # Run container with database environment variables
    sudo docker run -d \
        --name campsite-tracker \
        -p 8080:8080 \
        --restart unless-stopped \
        -e DATABASE_URL="postgres://postgres:$DB_PASSWORD@$DB_ENDPOINT/campsite_tracker" \
        -e JWT_SECRET="$(openssl rand -base64 32)" \
        -e RUST_LOG=info \
        campsite-tracker
        
    echo "Container started!"
    sudo docker ps
    
    # Check if container is healthy
    sleep 5
    if sudo docker ps | grep -q campsite-tracker; then
        echo "✅ Container is running!"
        echo "📋 Checking logs..."
        sudo docker logs campsite-tracker --tail 10
    else
        echo "❌ Container failed to start!"
        sudo docker logs campsite-tracker
        exit 1
    fi
EOF

# Cleanup
rm campsite-tracker.tar.gz

echo "✅ Deployment complete!"
echo "🌐 Your app is available at: http://$INSTANCE_IP:8080"
echo "📋 Check logs with: ssh -i campsite-key.pem ec2-user@$INSTANCE_IP 'sudo docker logs campsite-tracker'"
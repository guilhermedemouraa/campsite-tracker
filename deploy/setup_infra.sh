#!/bin/bash
set -e

echo "🚀 Creating AWS infrastructure..."

# Check if key pair already exists
if aws ec2 describe-key-pairs --key-names campsite-key >/dev/null 2>&1; then
    echo "Key pair already exists"
else
    echo "Creating key pair..."
    aws ec2 create-key-pair --key-name campsite-key --query 'KeyMaterial' --output text > campsite-key.pem
    chmod 400 campsite-key.pem
fi

# Create security group
SG_ID=$(aws ec2 create-security-group \
    --group-name campsite-sg-$(date +%s) \
    --description "Campsite app security group" \
    --query 'GroupId' \
    --output text 2>/dev/null || \
    aws ec2 describe-security-groups \
    --group-names campsite-sg \
    --query 'SecurityGroups[0].GroupId' \
    --output text)

echo "Security Group ID: $SG_ID"

# Allow SSH and HTTP (only if rules don't exist)
aws ec2 authorize-security-group-ingress \
    --group-id $SG_ID \
    --protocol tcp \
    --port 22 \
    --cidr 0.0.0.0/0 2>/dev/null || echo "SSH rule already exists"

aws ec2 authorize-security-group-ingress \
    --group-id $SG_ID \
    --protocol tcp \
    --port 8080 \
    --cidr 0.0.0.0/0 2>/dev/null || echo "HTTP rule already exists"

# Launch instance
INSTANCE_ID=$(aws ec2 run-instances \
    --image-id ami-05cb9251eea88c2ef \
    --count 1 \
    --instance-type t2.micro \
    --key-name campsite-key \
    --security-group-ids $SG_ID \
    --user-data '#!/bin/bash
yum update -y
yum install -y docker
systemctl start docker
systemctl enable docker
usermod -a -G docker ec2-user' \
    --query 'Instances[0].InstanceId' \
    --output text)

echo "✅ Infrastructure created!"
echo "Instance ID: $INSTANCE_ID"
echo "Security Group ID: $SG_ID"

# Save for later scripts
echo "INSTANCE_ID=$INSTANCE_ID" > .env
echo "SG_ID=$SG_ID" >> .env

echo ""
echo "⏳ Wait about 3 minutes for instance to boot and install Docker..."
echo "Then run: ./build-and-deploy.sh"
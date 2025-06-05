#!/bin/bash
set -e

echo "🗄️ Setting up PostgreSQL database on RDS..."

# Load environment
source .env

# Generate a random password
DB_PASSWORD=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-25)
echo "Generated database password: $DB_PASSWORD"

# Create DB subnet group (need at least 2 subnets in different AZs)
echo "Creating DB subnet group..."

# Get default VPC
VPC_ID=$(aws ec2 describe-vpcs --filters "Name=is-default,Values=true" --query 'Vpcs[0].VpcId' --output text)
echo "Using VPC: $VPC_ID"

# Get subnets in different AZs
SUBNET_IDS=$(aws ec2 describe-subnets \
    --filters "Name=vpc-id,Values=$VPC_ID" \
    --query 'Subnets[0:2].SubnetId' \
    --output text)

echo "Using subnets: $SUBNET_IDS"

# Create DB subnet group
aws rds create-db-subnet-group \
    --db-subnet-group-name campsite-db-subnet-group \
    --db-subnet-group-description "Campsite DB subnet group" \
    --subnet-ids $SUBNET_IDS 2>/dev/null || echo "DB subnet group already exists"

# Create database security group
DB_SG_ID=$(aws ec2 create-security-group \
    --group-name campsite-db-sg \
    --description "Campsite database security group" \
    --vpc-id $VPC_ID \
    --query 'GroupId' \
    --output text 2>/dev/null || \
    aws ec2 describe-security-groups \
    --group-names campsite-db-sg \
    --query 'SecurityGroups[0].GroupId' \
    --output text)

echo "Database Security Group ID: $DB_SG_ID"

# Allow PostgreSQL access from app security group
aws ec2 authorize-security-group-ingress \
    --group-id $DB_SG_ID \
    --protocol tcp \
    --port 5432 \
    --source-group $SG_ID 2>/dev/null || echo "DB security rule already exists"

# Create RDS instance
echo "Creating RDS PostgreSQL instance (this takes 5-10 minutes)..."
aws rds create-db-instance \
    --db-instance-identifier campsite-db \
    --db-instance-class db.t3.micro \
    --engine postgres \
    --engine-version 15.7 \
    --master-username postgres \
    --master-user-password "$DB_PASSWORD" \
    --allocated-storage 20 \
    --vpc-security-group-ids $DB_SG_ID \
    --db-subnet-group-name campsite-db-subnet-group \
    --backup-retention-period 7 \
    --storage-encrypted \
    --no-publicly-accessible \
    --db-name campsite_tracker || echo "RDS instance already exists"

echo "⏳ Waiting for database to become available..."
aws rds wait db-instance-available --db-instance-identifier campsite-db

# Get database endpoint
DB_ENDPOINT=$(aws rds describe-db-instances \
    --db-instance-identifier campsite-db \
    --query 'DBInstances[0].Endpoint.Address' \
    --output text)

echo "✅ Database created!"
echo "Database endpoint: $DB_ENDPOINT"

# Save database info
echo "DB_ENDPOINT=$DB_ENDPOINT" >> .env
echo "DB_PASSWORD=$DB_PASSWORD" >> .env

echo ""
echo "🔧 Next steps:"
echo "1. Run the migration: ./run_migration.sh"
echo "2. Deploy your app: ./build_and_deploy.sh"
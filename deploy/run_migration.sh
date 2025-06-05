#!/bin/bash
set -e

echo "🗄️ Running database migrations..."

# Load environment
source .env

echo "Connecting to database and running migrations..."

# Get instance IP for SSH tunnel
INSTANCE_IP=$(aws ec2 describe-instances \
    --instance-ids $INSTANCE_ID \
    --query 'Reservations[0].Instances[0].PublicIpAddress' \
    --output text)

# Copy migration file to EC2 instance
scp -i campsite-key.pem -o StrictHostKeyChecking=no \
    ../backend/migrations/001_initial_schema.sql ec2-user@$INSTANCE_IP:/home/ec2-user/

# Run migration through EC2 instance using Docker (which has access to RDS)
ssh -i campsite-key.pem -o StrictHostKeyChecking=no ec2-user@$INSTANCE_IP << EOF
    # Use Docker to run a PostgreSQL 15 client
    sudo docker run --rm \
        -v /home/ec2-user/001_initial_schema.sql:/migration.sql \
        postgres:15-alpine \
        psql "postgres://postgres:$DB_PASSWORD@$DB_ENDPOINT/campsite_tracker" \
        -f /migration.sql
    
    echo "✅ Migration completed!"
EOF

echo "✅ Database migration successful!"
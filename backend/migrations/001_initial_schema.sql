-- Campsite Tracker Database Schema
-- Migration 001: Initial tables

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    phone VARCHAR(20), -- E.164 format: +1234567890
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) DEFAULT 'user', -- user, admin, moderator
    email_verified BOOLEAN DEFAULT FALSE,
    phone_verified BOOLEAN DEFAULT FALSE,
    notification_preferences JSONB DEFAULT '{"email": true, "sms": true}',
    timezone VARCHAR(50) DEFAULT 'America/Los_Angeles',
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Campgrounds table (RIDB facilities)
CREATE TABLE campgrounds (
    id VARCHAR(50) PRIMARY KEY, -- RIDB facility ID
    name VARCHAR(255) NOT NULL,
    parent_recarea_id VARCHAR(50),
    parent_recarea_name VARCHAR(255),
    state VARCHAR(2),
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    total_sites INTEGER,
    is_reservable BOOLEAN DEFAULT TRUE,
    is_active BOOLEAN DEFAULT TRUE,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Campsites table (individual sites within campgrounds)
CREATE TABLE campsites (
    id VARCHAR(100) PRIMARY KEY, -- RIDB campsite ID
    campground_id VARCHAR(50) NOT NULL REFERENCES campgrounds(id),
    site_number VARCHAR(20),
    site_name VARCHAR(100),
    site_type VARCHAR(50), -- tent, rv, group, etc.
    max_occupancy INTEGER,
    is_accessible BOOLEAN DEFAULT FALSE,
    amenities JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT TRUE,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User scans table (what users want to monitor)
CREATE TABLE user_scans (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    campground_id VARCHAR(50) NOT NULL REFERENCES campgrounds(id),
    check_in_date DATE NOT NULL,
    check_out_date DATE NOT NULL,
    nights INTEGER GENERATED ALWAYS AS (check_out_date - check_in_date) STORED,
    status VARCHAR(20) DEFAULT 'active', -- active, paused, completed, cancelled
    notification_sent BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    
    CONSTRAINT valid_date_range CHECK (check_out_date > check_in_date),
    CONSTRAINT valid_expiry CHECK (expires_at IS NULL OR expires_at > created_at)
);

-- Campground availability cache
CREATE TABLE campground_availability (
    campground_id VARCHAR(50) NOT NULL REFERENCES campgrounds(id),
    date DATE NOT NULL,
    available_sites INTEGER DEFAULT 0,
    total_sites INTEGER DEFAULT 0,
    availability_data JSONB,
    last_checked TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    check_status VARCHAR(20) DEFAULT 'success', -- success, error, rate_limited
    error_message TEXT,
    
    PRIMARY KEY (campground_id, date)
);

-- Polling jobs for efficient monitoring
CREATE TABLE polling_jobs (
    campground_id VARCHAR(50) PRIMARY KEY REFERENCES campgrounds(id),
    active_scan_count INTEGER DEFAULT 0,
    last_polled TIMESTAMP WITH TIME ZONE,
    next_poll_at TIMESTAMP WITH TIME ZONE,
    poll_frequency_minutes INTEGER DEFAULT 15,
    consecutive_errors INTEGER DEFAULT 0,
    is_being_polled BOOLEAN DEFAULT FALSE,
    priority INTEGER DEFAULT 1,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Notifications audit trail
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    user_scan_id UUID REFERENCES user_scans(id) ON DELETE SET NULL,
    type VARCHAR(20) NOT NULL, -- sms, email
    recipient VARCHAR(255) NOT NULL,
    subject VARCHAR(255),
    message TEXT NOT NULL,
    availability_details JSONB,
    status VARCHAR(20) DEFAULT 'pending', -- pending, sent, failed, delivered
    sent_at TIMESTAMP WITH TIME ZONE,
    external_id VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User sessions for JWT refresh tokens
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    user_agent TEXT,
    ip_address INET,
    is_revoked BOOLEAN DEFAULT FALSE
);

-- Indexes for performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_user_scans_user_id ON user_scans(user_id);
CREATE INDEX idx_user_scans_campground_dates ON user_scans(campground_id, check_in_date, check_out_date);
CREATE INDEX idx_user_scans_active ON user_scans(status) WHERE status = 'active';
CREATE INDEX idx_polling_jobs_next_poll ON polling_jobs(next_poll_at);
CREATE INDEX idx_campground_availability_campground_date ON campground_availability(campground_id, date);
CREATE INDEX idx_notifications_user_scan ON notifications(user_scan_id);
CREATE INDEX idx_notifications_status ON notifications(status);
CREATE INDEX idx_campsites_campground ON campsites(campground_id);

-- Trigger function to update polling job counts
CREATE OR REPLACE FUNCTION update_polling_job_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO polling_jobs (campground_id, active_scan_count, next_poll_at)
        VALUES (NEW.campground_id, 1, NOW())
        ON CONFLICT (campground_id) 
        DO UPDATE SET 
            active_scan_count = polling_jobs.active_scan_count + 1,
            next_poll_at = LEAST(polling_jobs.next_poll_at, NOW());
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE polling_jobs 
        SET active_scan_count = GREATEST(active_scan_count - 1, 0)
        WHERE campground_id = OLD.campground_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically manage polling jobs
CREATE TRIGGER trigger_update_polling_count
    AFTER INSERT OR DELETE ON user_scans
    FOR EACH ROW EXECUTE FUNCTION update_polling_job_count();

# 🏕️ Campsite Tracker

A comprehensive campsite availability tracker for National Parks with real-time monitoring, user authentication, and SMS/email notifications.

## ✨ Features

### 🔍 **Smart Campground Search**

- Search National Parks campgrounds using Recreation.gov RIDB API
- Filter by location, amenities, and availability
- Real-time facility information from official sources

### 👤 **User Management**

- Secure user registration and authentication with JWT tokens
- Email and SMS verification
- User profiles with notification preferences
- Password hashing with bcrypt

### � **Scan Management**

- Create custom campground availability scans
- Monitor specific date ranges for availability
- Track multiple campgrounds simultaneously
- Pause, resume, or cancel scans as needed

### 📱 **Multi-Channel Notifications**

- SMS alerts via AWS SNS for immediate notifications
- Email notifications with verification links
- Customizable notification preferences per user

### 🎨 **Beautiful UI**

- Mountain-themed React interface with TypeScript
- Responsive design for mobile and desktop
- Intuitive scan creation and management
- Real-time status updates

### 🐳 **Production Ready**

- Containerized with Docker for easy deployment
- PostgreSQL database with migrations
- AWS deployment scripts included
- Comprehensive error handling and logging

## 🛠️ Tech Stack

### Frontend

- **React 19** with TypeScript
- **Lucide React** for icons
- **CSS3** with custom mountain theme
- **Responsive design** for all devices

### Backend (Rust Workspace)

- **Actix Web** for HTTP server
- **SQLx** with PostgreSQL for database operations
- **JWT** for authentication
- **bcrypt** for password hashing
- **AWS SNS** for SMS notifications
- **Email services** for verification

### Backend Architecture (Modular Crates)

- `web_server` - Main HTTP server and routing
- `auth_services` - User authentication and JWT handling
- `campground-scan` - Scan management and database operations
- `notification_services` - SMS and email notification handling
- `rec_gov` - Recreation.gov API integration
- `postgres` - Database connection and utilities
- `web_handlers` - HTTP request handlers organized by domain

### Infrastructure

- **PostgreSQL** for persistent data storage
- **Docker** for containerization
- **AWS EC2** for hosting
- **AWS SNS** for SMS delivery
- **Recreation.gov RIDB API** for campground data

## 🏗️ Project Structure

```
campsite-tracker/
├── backend/                    # Rust workspace
│   ├── crates/
│   │   ├── web_server/        # Main HTTP server
│   │   ├── auth_services/     # Authentication & JWT
│   │   ├── campground-scan/   # Scan management
│   │   ├── notification_services/ # SMS & Email
│   │   ├── rec_gov/          # Recreation.gov API
│   │   ├── postgres/         # Database utilities
│   │   └── web_handlers/     # HTTP handlers
│   │       ├── auth_handlers.rs      # Signup, login
│   │       ├── profile_handlers.rs   # User profiles
│   │       ├── verification_handlers.rs # Email/SMS verification
│   │       ├── admin_handlers.rs     # Admin/dev endpoints
│   │       └── scan_handlers.rs      # Scan CRUD operations
│   ├── migrations/           # Database migrations
│   └── Cargo.toml           # Workspace configuration
├── frontend/                 # React TypeScript app
│   └── src/
│       ├── components/
│       │   ├── Auth/         # Login/signup forms
│       │   ├── CreateScan/   # Scan creation interface
│       │   ├── UserProfile/  # Profile management
│       │   ├── UserScans/    # Scan list and management
│       │   ├── FacilitySearch/ # Campground search
│       │   └── DatePicker/   # Date selection
│       └── ...
├── deploy/                   # AWS deployment scripts
├── py/                      # Python utilities (legacy)
└── Dockerfile              # Multi-stage container build
```

## 🚀 Getting Started

### Prerequisites

- **Node.js 18+** for frontend development
- **Rust nightly** for backend (requires Edition 2024)
- **PostgreSQL** for database
- **Docker** for deployment
- **AWS Account** for SMS notifications (optional for development)

### Environment Setup

1. **Clone the repository**

   ```bash
   git clone https://github.com/YOUR_USERNAME/campsite-tracker.git
   cd campsite-tracker
   ```

2. **Database Setup**

   ```bash
   # Install and start PostgreSQL
   brew install postgresql@16
   brew services start postgresql@16

   # Create database
   createdb campsite_tracker
   ```

3. **Backend Environment**
   ```bash
   cd backend
   cp .env.example .env
   # Edit .env with your database URL and AWS credentials
   ```

### Local Development

1. **Start the Backend**

   ```bash
   cd backend
   cargo run --bin web_server
   ```

   The API will be available at `http://localhost:8080`

2. **Start the Frontend**

   ```bash
   cd frontend
   npm install
   npm start
   ```

   The app will be available at `http://localhost:3000`

3. **Access the Application**
   - Frontend: `http://localhost:3000`
   - Backend API: `http://localhost:8080/api`
   - Health Check: `http://localhost:8080/health`

## 🔧 API Endpoints

### Authentication

- `POST /api/auth/signup` - User registration
- `POST /api/auth/login` - User login
- `GET /api/auth/health` - Auth service health check

### User Management

- `GET /api/user/profile` - Get user profile
- `PUT /api/user/profile/update` - Update user profile
- `POST /api/user/verify/email/send` - Send email verification
- `POST /api/user/verify/sms/send` - Send SMS verification
- `POST /api/user/verify/sms` - Verify SMS code

### Scan Management

- `POST /api/scans` - Create new scan
- `GET /api/scans` - Get user's scans
- `GET /api/scans/active` - Get active scans only
- `GET /api/scans/{id}` - Get specific scan
- `PUT /api/scans/{id}` - Update scan status
- `DELETE /api/scans/{id}` - Delete scan

### Campground Search

- `GET /api/facilities/search?q={query}` - Search campgrounds

## 🐳 Deployment

### Docker Build

```bash
docker build -t campsite-tracker .
docker run -p 8080:8080 campsite-tracker
```

### AWS Deployment

```bash
cd deploy
./setup_infra.sh      # First time setup
./build_and_deploy.sh # Deploy updates
```

## 🔒 Security Features

- **JWT Authentication** with refresh tokens
- **Password hashing** using bcrypt
- **Email verification** with secure tokens
- **SMS verification** with time-limited codes
- **Input validation** on all endpoints
- **SQL injection protection** with SQLx
- **HTTPS ready** for production deployment

## 📱 Notification System

- **SMS Notifications**: Powered by AWS SNS
- **Email Notifications**: HTML templates with verification links
- **User Preferences**: Configurable per-user notification settings
- **Rate Limiting**: Prevents spam and abuse

## 🧪 Development

### Testing

```bash
cd backend
cargo test

cd frontend
npm test
```

### Code Organization

- **Modular Backend**: Each domain has its own crate
- **Type Safety**: Full TypeScript on frontend, Rust on backend
- **Error Handling**: Comprehensive error types and responses
- **Logging**: Structured logging throughout the application

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **Recreation.gov** for providing the RIDB API
- **National Park Service** for maintaining campground data
- **Rust Community** for excellent web development tools
- **React Community** for the frontend ecosystem

```

```

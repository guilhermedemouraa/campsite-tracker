# 🏕️ Campsite Tracker

A real-time campsite availability tracker for National Parks with SMS notifications.

## ✨ Features

- 🔍 **Smart Search**: Find campgrounds in National Parks (Yosemite, Sequoia, etc.)
- 🏔️ **Beautiful UI**: Mountain-themed React interface
- 📱 **SMS Alerts**: Get notified when your dream campsite becomes available
- ⚡ **Real-time**: Uses RIDB API for up-to-date campground information
- 🐳 **Containerized**: Docker deployment ready for AWS

## 🛠️ Tech Stack

- **Frontend**: React + TypeScript with mountain-themed UI
- **Backend**: Rust + Actix Web
- **Database**: SQLite (planned)
- **SMS**: AWS SNS / Twilio
- **Deployment**: Docker + AWS EC2
- **API**: Recreation.gov RIDB API

## 🚀 Getting Started

### Prerequisites

- Node.js 18+
- Rust 1.83+
- Docker (for deployment)

### Local Development

1. **Clone the repository**

   ```bash
   git clone https://github.com/YOUR_USERNAME/campsite-tracker.git
   cd campsite-tracker
   ```

2. **Start the frontend**

   ```bash
   cd frontend
   npm install
   npm start
   ```

3. **Start the backend**

   ```bash
   cd ../backend
   cargo run
   ```

4. **Access the app**

   Open your browser and go to `http://localhost:3000` for the frontend and `http://localhost:8080` for the backend API.

5. **Deploy to AWS**

   Make sure you have Docker installed and configured for AWS. Then run the following commands:

   ```bash
   cd deploy
   ./setup_infra.sh # First time only
   ./build_and_deploy.sh # Deploy updates
   ```

```

```

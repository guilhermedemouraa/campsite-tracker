import React from 'react';
import Header from './components/Header/Header';
import CreateScan from './components/CreateScan/CreateScan';
import MountainBackground from './components/MountainBackground/MountainBackground';
import './App.css';

const App: React.FC = () => {
  return (
    <div className="app">
      <MountainBackground />
      <div className="app-content">
        <Header />
        <main className="main">
          <div className="hero">
            <h1 className="hero-title">Never Miss a Campsite</h1>
            <p className="hero-subtitle">
              Get notified instantly when your dream campsite becomes available
            </p>
          </div>
          <CreateScan />
        </main>
      </div>
    </div>
  );
};

export default App;
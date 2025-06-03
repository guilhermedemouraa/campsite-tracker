import React from 'react';
import { User, MapPin } from 'lucide-react';
import './Header.css';

const Header: React.FC = () => {
  const handleSignIn = () => {
    // TODO: Implement sign-in functionality
    console.log('Sign in clicked');
  };

  const handleSignUp = () => {
    // TODO: Implement sign-up functionality  
    console.log('Sign up clicked');
  };

  return (
    <header className="header">
      <div className="header-content">
        <div className="logo">
          <MapPin className="logo-icon" />
          <span className="logo-text">CampTracker</span>
        </div>
        <nav className="nav">
          <button className="nav-button" onClick={handleSignIn}>
            <User size={20} />
            Sign In
          </button>
          <button className="nav-button primary" onClick={handleSignUp}>
            Sign Up
          </button>
        </nav>
      </div>
    </header>
  );
};

export default Header;
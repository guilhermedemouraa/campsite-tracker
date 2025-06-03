import React, { useState } from 'react';
import { Plus } from 'lucide-react';
import FacilitySearch, { Facility } from '../FacilitySearch/FacilitySearch';
import DatePicker from '../DatePicker/DatePicker';
import './CreateScan.css';

const CreateScan: React.FC = () => {
  const [facilitySearch, setFacilitySearch] = useState<string>('');
  const [selectedFacility, setSelectedFacility] = useState<Facility | null>(null);
  const [checkIn, setCheckIn] = useState<string>('');
  const [checkOut, setCheckOut] = useState<string>('');

  const handleFacilityChange = (value: string) => {
    setFacilitySearch(value);
    // Clear selected facility if user starts typing again
    if (selectedFacility && value !== selectedFacility.name) {
      setSelectedFacility(null);
    }
  };

  const handleFacilitySelect = (facility: Facility) => {
    setSelectedFacility(facility);
    console.log('Selected facility:', facility);
  };

  const handleCreateScan = () => {
    if (!facilitySearch || !checkIn || !checkOut) {
      alert('Please fill in all fields');
      return;
    }
    
    if (new Date(checkOut) <= new Date(checkIn)) {
      alert('Check-out date must be after check-in date');
      return;
    }
    
    console.log('Creating scan:', {
      facility: selectedFacility || { name: facilitySearch },
      checkIn,
      checkOut
    });
    
    // TODO: Implement actual scan creation logic
    alert(`Scan would be created for ${facilitySearch} from ${checkIn} to ${checkOut}`);
  };

  const isFormValid = facilitySearch && checkIn && checkOut && new Date(checkOut) > new Date(checkIn);

  return (
    <div className="create-scan">
      <div className="scan-card">
        <div className="scan-header">
          <Plus className="scan-icon" />
          <h2 className="scan-title">Create a New Scan</h2>
          <p className="scan-subtitle">Monitor campsite availability for your trip</p>
        </div>
        
        <div className="scan-form">
          <div className="form-group">
            <label className="form-label">
              Facility
              {selectedFacility && (
                <span className="selected-indicator"> ✓ Selected</span>
              )}
            </label>
            <FacilitySearch
              value={facilitySearch}
              onChange={handleFacilityChange}
              onFacilitySelect={handleFacilitySelect}
            />
          </div>
          
          <div className="form-group">
            <label className="form-label">Dates</label>
            <DatePicker
              checkIn={checkIn}
              checkOut={checkOut}
              onCheckInChange={setCheckIn}
              onCheckOutChange={setCheckOut}
            />
          </div>
          
          <button 
            className={`create-button ${!isFormValid ? 'disabled' : ''}`}
            onClick={handleCreateScan}
            disabled={!isFormValid}
          >
            <Plus size={20} />
            Create Scan
          </button>
        </div>
      </div>
    </div>
  );
};

export default CreateScan;
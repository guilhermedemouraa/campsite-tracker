import React, { useState } from "react";
import { Plus } from "lucide-react";
import FacilitySearch, { Facility } from "../FacilitySearch/FacilitySearch";
import DatePicker from "../DatePicker/DatePicker";
import { createScan, formatDateForApi } from "./ScanUtils";
import "./CreateScan.css";

const CreateScan: React.FC = () => {
  const [facilitySearch, setFacilitySearch] = useState<string>("");
  const [selectedFacility, setSelectedFacility] = useState<Facility | null>(
    null,
  );
  const [checkIn, setCheckIn] = useState<string>("");
  const [checkOut, setCheckOut] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(false);

  const handleFacilityChange = (value: string) => {
    setFacilitySearch(value);
    // Clear selected facility if user starts typing again
    if (selectedFacility && value !== selectedFacility.name) {
      setSelectedFacility(null);
    }
  };

  const handleFacilitySelect = (facility: Facility) => {
    setSelectedFacility(facility);
    console.log("Selected facility:", facility);
  };

  const handleCreateScan = async () => {
    if (!facilitySearch || !checkIn || !checkOut) {
      alert("Please fill in all fields");
      return;
    }

    if (new Date(checkOut) <= new Date(checkIn)) {
      alert("Check-out date must be after check-in date");
      return;
    }

    if (!selectedFacility) {
      alert("Please select a facility from the search results");
      return;
    }

    setIsLoading(true);

    try {
      const scanData = {
        campground_id: selectedFacility.id.toString(),
        campground_name: selectedFacility.name,
        check_in_date: formatDateForApi(checkIn),
        check_out_date: formatDateForApi(checkOut),
      };

      console.log("Creating scan:", scanData);

      const result = await createScan(scanData);

      console.log("Scan created successfully:", result);

      // Reset form
      setFacilitySearch("");
      setSelectedFacility(null);
      setCheckIn("");
      setCheckOut("");

      alert(
        `Scan created successfully! Monitoring ${result.campground_name} for ${result.nights} nights.`,
      );
    } catch (error) {
      console.error("Error creating scan:", error);
      alert(
        error instanceof Error
          ? `Failed to create scan: ${error.message}`
          : "Failed to create scan. Please try again.",
      );
    } finally {
      setIsLoading(false);
    }
  };

  const isFormValid =
    facilitySearch &&
    selectedFacility &&
    checkIn &&
    checkOut &&
    new Date(checkOut) > new Date(checkIn) &&
    !isLoading;

  return (
    <div className="create-scan">
      <div className="create-scan-card">
        <div className="create-scan-header">
          <Plus className="create-scan-icon" />
          <h2 className="create-scan-title">Create a New Scan</h2>
          <p className="create-scan-subtitle">
            Monitor campsite availability for your trip
          </p>
        </div>

        <div className="create-scan-form">
          <div className="create-scan-form-group">
            <label className="create-scan-form-label">
              Facility
              {selectedFacility && (
                <span className="create-scan-selected-indicator">
                  {" "}
                  ✓ Selected
                </span>
              )}
            </label>
            <FacilitySearch
              value={facilitySearch}
              onChange={handleFacilityChange}
              onFacilitySelect={handleFacilitySelect}
            />
          </div>

          <div className="create-scan-form-group">
            <label className="create-scan-form-label">Dates</label>
            <DatePicker
              checkIn={checkIn}
              checkOut={checkOut}
              onCheckInChange={setCheckIn}
              onCheckOutChange={setCheckOut}
            />
          </div>

          <button
            className={`create-scan-button ${!isFormValid ? "disabled" : ""}`}
            onClick={handleCreateScan}
            disabled={!isFormValid}
          >
            <Plus size={20} />
            {isLoading ? "Creating..." : "Create Scan"}
          </button>
        </div>
      </div>
    </div>
  );
};

export default CreateScan;

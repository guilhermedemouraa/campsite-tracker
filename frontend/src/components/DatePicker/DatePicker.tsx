import React, { useRef } from "react";
import { Calendar } from "lucide-react";
import "./DatePicker.css";

interface DatePickerProps {
  checkIn: string;
  checkOut: string;
  onCheckInChange: (date: string) => void;
  onCheckOutChange: (date: string) => void;
}

const DatePicker: React.FC<DatePickerProps> = ({
  checkIn,
  checkOut,
  onCheckInChange,
  onCheckOutChange,
}) => {
  const checkInRef = useRef<HTMLInputElement>(null);
  const checkOutRef = useRef<HTMLInputElement>(null);

  // Get today's date in YYYY-MM-DD format for min attribute
  const today = new Date().toISOString().split("T")[0];

  const handleCheckInClick = () => {
    checkInRef.current?.showPicker?.();
  };

  const handleCheckOutClick = () => {
    checkOutRef.current?.showPicker?.();
  };

  return (
    <div className="date-picker">
      <div className="date-input-group">
        <label className="date-label">Check-in</label>
        <div className="date-input-wrapper" onClick={handleCheckInClick}>
          <Calendar className="date-icon" />
          <input
            ref={checkInRef}
            type="date"
            value={checkIn}
            min={today}
            onChange={(e) => onCheckInChange(e.target.value)}
            className="date-input"
            placeholder="Select check-in date"
          />
        </div>
      </div>
      <div className="date-input-group">
        <label className="date-label">Check-out</label>
        <div className="date-input-wrapper" onClick={handleCheckOutClick}>
          <Calendar className="date-icon" />
          <input
            ref={checkOutRef}
            type="date"
            value={checkOut}
            min={checkIn || today}
            onChange={(e) => onCheckOutChange(e.target.value)}
            className="date-input"
            placeholder="Select check-out date"
          />
        </div>
      </div>
    </div>
  );
};

export default DatePicker;

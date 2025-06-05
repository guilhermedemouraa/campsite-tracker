import React, { useState, useEffect, useCallback } from "react";
import { Search, MapPin } from "lucide-react";
import "./FacilitySearch.css";

export interface Facility {
  id: number;
  name: string;
  description?: string;
  state?: string;
}

interface FacilitySearchProps {
  value: string;
  onChange: (value: string) => void;
  onFacilitySelect: (facility: Facility) => void;
  placeholder?: string;
}

// Cache for API results
const facilityCache = new Map<string, Facility[]>();

const searchFacilities = async (query: string): Promise<Facility[]> => {
  // Check cache first
  const cacheKey = query.toLowerCase();
  if (facilityCache.has(cacheKey)) {
    return facilityCache.get(cacheKey)!;
  }

  try {
    const response = await fetch(
      `/api/facilities/search?q=${encodeURIComponent(query)}`,
    );

    if (!response.ok) {
      throw new Error(`API error: ${response.status}`);
    }

    const data = await response.json();
    console.log("API Response for query:", query, data); // Debug log

    const facilities: Facility[] =
      data.RECDATA?.map((facility: any) => ({
        id: facility.FacilityID,
        name: facility.FacilityName,
        description: facility.FacilityDescription,
        state: facility.AddressStateCode,
      })) || [];

    console.log("Parsed facilities:", facilities); // Debug log

    // Cache the results
    facilityCache.set(cacheKey, facilities);

    return facilities;
  } catch (error) {
    console.error("Error searching facilities:", error);
    return [];
  }
};

// Simple debounce function
function debounce<T extends (...args: any[]) => void>(
  func: T,
  wait: number,
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout;
  return (...args: Parameters<T>) => {
    clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}

const FacilitySearch: React.FC<FacilitySearchProps> = ({
  value,
  onChange,
  onFacilitySelect,
  placeholder = "Search campgrounds (e.g., North Pines, Upper Pines)",
}) => {
  const [isLoading, setIsLoading] = useState(false);
  const [suggestions, setSuggestions] = useState<Facility[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);

  // Debounced search function
  const debouncedSearch = useCallback(
    debounce(async (query: string) => {
      if (query.length >= 2) {
        // Lowered from 3 to 2
        setIsLoading(true);
        try {
          const results = await searchFacilities(query);
          setSuggestions(results);
        } catch (error) {
          console.error("Search failed:", error);
          setSuggestions([]);
        }
        setIsLoading(false);
      } else {
        setSuggestions([]);
      }
    }, 300),
    [],
  );

  useEffect(() => {
    debouncedSearch(value);
    setShowSuggestions(value.length > 0);
  }, [value, debouncedSearch]);

  const handleSuggestionClick = (facility: Facility) => {
    onChange(facility.name);
    onFacilitySelect(facility);
    setShowSuggestions(false);
  };

  const handleInputChange = (newValue: string) => {
    onChange(newValue);
    setShowSuggestions(true);
  };

  const handleInputBlur = () => {
    setTimeout(() => setShowSuggestions(false), 150);
  };

  return (
    <div className="search-container">
      <div className="search-input-wrapper">
        <Search className={`search-icon ${isLoading ? "loading" : ""}`} />
        <input
          type="text"
          placeholder={placeholder}
          value={value}
          onChange={(e) => handleInputChange(e.target.value)}
          onFocus={() => setShowSuggestions(value.length > 0)}
          onBlur={handleInputBlur}
          className="search-input"
        />
        {isLoading && <div className="loading-spinner">⟳</div>}
      </div>
      {showSuggestions && suggestions.length > 0 && (
        <div className="suggestions">
          {suggestions.map((suggestion) => (
            <div
              key={suggestion.id}
              className="suggestion-item"
              onMouseDown={() => handleSuggestionClick(suggestion)}
            >
              <MapPin size={16} />
              <div className="suggestion-content">
                <span className="suggestion-name">{suggestion.name}</span>
                {suggestion.state && (
                  <span className="suggestion-state">{suggestion.state}</span>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
      {showSuggestions &&
        value.length >= 2 &&
        suggestions.length === 0 &&
        !isLoading && (
          <div className="suggestions">
            <div className="no-results">No campgrounds found for "{value}"</div>
          </div>
        )}
    </div>
  );
};

export default FacilitySearch;

#!/usr/bin/env python3
"""
Yosemite Campsite Monitor using the recgov library approach
Based on the source code you provided
"""

import requests
import os
from datetime import datetime, timedelta
from typing import Dict, List, Optional
import time
import json
from fake_useragent import UserAgent


class SessionPaginator(requests.Session):
    """Session with pagination support for RIDB API"""

    def get_record_iterator(self, *args, params=None, **kwargs):
        params = params or {}
        total_count = None
        current_count = 0
        while not (current_count == total_count):
            params.update({"limit": 50, "offset": current_count})
            resp = self.get(*args, params=params, **kwargs).json()
            for rec in resp["RECDATA"]:
                yield rec
            current_count += resp["METADATA"]["RESULTS"]["CURRENT_COUNT"]
            total_count = resp["METADATA"]["RESULTS"]["TOTAL_COUNT"]
            if current_count > total_count:
                raise ValueError(
                    f"Total records was supposed to be "
                    f"{total_count}, but we somehow read "
                    f"{current_count}!"
                )


def get_session(apikey: str = None) -> SessionPaginator:
    """Gets a session with an apikey for RIDB API"""
    if apikey is None and "RECREATION_GOV_KEY" in os.environ:
        apikey = os.environ.get("RECREATION_GOV_KEY")
    if apikey is None:
        raise RuntimeError(
            "apikey must be provided either as parameter or RECREATION_GOV_KEY env var"
        )

    headers = {"apikey": apikey}
    sess = SessionPaginator()
    sess.headers.update(headers)
    return sess


def get_anonymous_session():
    """Gets anonymous session for Recreation.gov internal API"""
    HEADERS = {"User-Agent": UserAgent().random}
    sess = requests.Session()
    sess.headers.update(HEADERS)
    return sess


class Campsite(dict):
    """Campsite object with useful properties"""

    @property
    def id(self):
        return self["CampsiteID"]

    @property
    def name(self):
        return self["CampsiteName"]

    @property
    def site_type(self):
        return self["CampsiteType"]

    @property
    def loop(self):
        return self["Loop"]

    @property
    def site_url(self):
        return f"https://www.recreation.gov/camping/campsites/{self.id}"

    @property
    def availabilities(self):
        """Convert availability data to readable date ranges"""
        if "availabilities" not in self:
            return []

        available_dates = []
        for date_str, status in self["availabilities"].items():
            if status == "Available":
                # Convert from ISO format to date
                date_obj = datetime.fromisoformat(date_str[:10])
                available_dates.append(date_obj)

        if not available_dates:
            return []

        # Sort dates
        available_dates.sort()

        # Group consecutive dates
        ranges = []
        start = available_dates[0]
        end = available_dates[0]

        for date in available_dates[1:]:
            if date == end + timedelta(days=1):
                end = date
            else:
                if start == end:
                    ranges.append(start.strftime("%Y-%m-%d"))
                else:
                    ranges.append(
                        f"{start.strftime('%Y-%m-%d')} to {end.strftime('%Y-%m-%d')}"
                    )
                start = end = date

        # Add the last range
        if start == end:
            ranges.append(start.strftime("%Y-%m-%d"))
        else:
            ranges.append(f"{start.strftime('%Y-%m-%d')} to {end.strftime('%Y-%m-%d')}")

        return ranges

    def has_consecutive_availability(self, start_date: datetime, nights: int) -> bool:
        """Check if site has consecutive availability for the specified period"""
        if "availabilities" not in self:
            return False

        for i in range(nights):
            check_date = start_date + timedelta(days=i)
            date_str = check_date.strftime("%Y-%m-%dT00:00:00Z")
            if self["availabilities"].get(date_str) != "Available":
                return False
        return True

    @property
    def available_nights(self) -> int:
        """Count available nights"""
        if "availabilities" not in self:
            return 0
        return len([v for v in self["availabilities"].values() if v == "Available"])


class CampsiteSet(dict):
    """Collection of campsites"""

    def ingest_availability(self, availability: dict) -> None:
        """Add availability data to campsites"""
        for campsite_id, avail_data in availability.items():
            if campsite_id in self:
                self[campsite_id]["availabilities"] = avail_data["availabilities"]

    def with_availability(
        self, start_date: datetime = None, nights: int = 1
    ) -> "CampsiteSet":
        """Return only campsites with availability for the specified period"""
        if start_date is None:
            # Original behavior - any availability
            return CampsiteSet(
                {k: v for k, v in self.items() if v.available_nights > 0}
            )
        else:
            # Check for consecutive availability
            return CampsiteSet(
                {
                    k: v
                    for k, v in self.items()
                    if v.has_consecutive_availability(start_date, nights)
                }
            )


class Availability(dict):
    """Availability checker using Recreation.gov internal API"""

    def __init__(self, asset_id: int):
        self.asset_id = asset_id
        self.sess = get_anonymous_session()
        super().__init__()

    def get_month_dict(self, month: datetime) -> dict:
        """Get availability data for a specific month"""
        # Ensure month is first day at midnight
        month = datetime(month.year, month.month, 1)

        url = f"https://www.recreation.gov/api/camps/availability/campground/{self.asset_id}/month"
        params = {"start_date": month.isoformat() + ".000Z"}

        print(f"🔍 Checking availability for {month.strftime('%B %Y')}")
        print(f"URL: {url}?start_date={params['start_date']}")

        try:
            resp = self.sess.get(url, params=params, timeout=15)
            print(f"Response status: {resp.status_code}")

            if resp.status_code == 200:
                data = resp.json()
                print(
                    f"✅ Successfully retrieved data for {len(data.get('campsites', {}))} campsites"
                )
                return data
            else:
                print(f"❌ Error: {resp.status_code} - {resp.text[:200]}")
                return {}

        except requests.RequestException as e:
            print(f"❌ Request failed: {e}")
            return {}

    def retrieve_month(self, month: datetime) -> None:
        """Retrieve and merge availability for a month"""
        month_data = self.get_month_dict(month)
        if month_data and "campsites" in month_data:
            for campsite_id, campsite_data in month_data["campsites"].items():
                if campsite_id in self:
                    if "availabilities" in self[campsite_id]:
                        self[campsite_id]["availabilities"].update(
                            campsite_data["availabilities"]
                        )
                    else:
                        self[campsite_id] = campsite_data
                else:
                    self[campsite_id] = campsite_data

    def apply_filters(self, filters: dict):
        """Apply date filters and retrieve availability"""
        if "start_date" not in filters:
            raise RuntimeError("start_date filter is required!")

        start_date = filters["start_date"]
        end_date = filters.get("end_date", start_date + timedelta(days=30))

        # Get all months in date range
        current_month = datetime(start_date.year, start_date.month, 1)
        end_month = datetime(end_date.year, end_date.month, 1)

        while current_month <= end_month:
            self.retrieve_month(current_month)
            # Move to next month
            if current_month.month == 12:
                current_month = datetime(current_month.year + 1, 1, 1)
            else:
                current_month = datetime(current_month.year, current_month.month + 1, 1)
            time.sleep(1)  # Be respectful to the API

        # Filter by date range
        filtered = {}
        for campsite_id, campsite_data in self.items():
            if "availabilities" in campsite_data:
                filtered_avail = {}
                for date_str, status in campsite_data["availabilities"].items():
                    date_obj = datetime.fromisoformat(date_str[:19])  # Remove timezone
                    # Convert both to dates for comparison
                    if start_date.date() <= date_obj.date() <= end_date.date():
                        filtered_avail[date_str] = status

                if filtered_avail:
                    campsite_copy = campsite_data.copy()
                    campsite_copy["availabilities"] = filtered_avail
                    filtered[campsite_id] = campsite_copy

        return filtered


def get_campsites(asset_id: int, apikey: str = None) -> CampsiteSet:
    """Get campsites using RIDB API"""
    sess = get_session(apikey=apikey)

    print(f"🏕️ Fetching campsite information for facility {asset_id}...")

    try:
        campsites = []
        for campsite in sess.get_record_iterator(
            f"https://ridb.recreation.gov/api/v1/facilities/{asset_id}/campsites"
        ):
            campsites.append(campsite)

        print(f"✅ Found {len(campsites)} campsites")
        return CampsiteSet({cs["CampsiteID"]: Campsite(cs) for cs in campsites})

    except Exception as e:
        print(f"❌ Error fetching campsites: {e}")
        return CampsiteSet({})


class YosemiteAvailabilityMonitor:
    """Monitor Yosemite campsite availability"""

    def __init__(self, ridb_api_key: str):
        self.ridb_api_key = ridb_api_key
        self.yosemite_campgrounds = {
            "Lodgepole": 232461,
            "Upper Pines": 232447,
            "Lower Pines": 232450,
            "North Pines": 232449,
        }

    def check_availability(
        self, campground_name: str, start_date: datetime, end_date: datetime
    ):
        """Check availability for a specific campground and date range"""
        if campground_name not in self.yosemite_campgrounds:
            print(f"❌ Unknown campground: {campground_name}")
            return

        asset_id = self.yosemite_campgrounds[campground_name]

        print(f"\n{'='*60}")
        print(f"🏔️ CHECKING {campground_name.upper()} (ID: {asset_id})")
        print(
            f"📅 Dates: {start_date.strftime('%Y-%m-%d')} to {end_date.strftime('%Y-%m-%d')}"
        )
        print(f"{'='*60}")

        # Get campsite information
        campsites = get_campsites(asset_id, self.ridb_api_key)

        if not campsites:
            print("❌ Could not retrieve campsite information")
            return

        # Get availability
        availability = Availability(asset_id)
        filtered_availability = availability.apply_filters(
            {"start_date": start_date, "end_date": end_date}
        )

        # Merge availability with campsite data
        campsites.ingest_availability(filtered_availability)

        # Get sites with consecutive availability for the full period
        nights = (end_date - start_date).days
        available_sites = campsites.with_availability(start_date, nights)

        if available_sites:
            print(
                f"\n🎉 FOUND {len(available_sites)} SITES WITH {nights}-NIGHT AVAILABILITY!"
            )
            print("-" * 50)

            for site_id, site in sorted(available_sites.items()):
                print(f"🏕️ Site: {site.name} (ID: {site.id})")
                print(f"   Type: {site.site_type}")
                print(f"   Loop: {site.loop}")
                print(
                    f"   Available for: {start_date.strftime('%Y-%m-%d')} to {end_date.strftime('%Y-%m-%d')} ({nights} nights)"
                )
                print(f"   All availability: {', '.join(site.availabilities)}")
                print(f"   Book at: {site.site_url}")
                print()
        else:
            print(f"\n❌ No {nights}-night availability found for {campground_name}")
            print(
                f"   Dates requested: {start_date.strftime('%Y-%m-%d')} to {end_date.strftime('%Y-%m-%d')}"
            )

            # Show partial availability for debugging
            partial_sites = campsites.with_availability()  # Any availability
            if partial_sites:
                print(
                    f"   ℹ️ However, {len(partial_sites)} sites have some availability in this period:"
                )
                for site_id, site in list(partial_sites.items())[:3]:  # Show first 3
                    print(f"      • {site.name}: {', '.join(site.availabilities)}")
                if len(partial_sites) > 3:
                    print(f"      • ... and {len(partial_sites) - 3} more sites")

    def monitor_date_ranges(self, date_ranges: List[tuple]):
        """Monitor all campgrounds for specific date ranges"""
        print("🏔️ YOSEMITE CAMPSITE AVAILABILITY MONITOR")
        print("=" * 70)

        # Check each campground for each date range
        for campground_name in self.yosemite_campgrounds:
            for start_date_str, end_date_str in date_ranges:
                start_date = datetime.strptime(start_date_str, "%Y-%m-%d")
                end_date = datetime.strptime(end_date_str, "%Y-%m-%d")
                self.check_availability(campground_name, start_date, end_date)
                time.sleep(2)  # Be respectful between requests

    def monitor_all_campgrounds(self, target_dates: List[str], nights: int = 1):
        """Monitor all Yosemite campgrounds for specific dates with configurable stay length"""
        print("🏔️ YOSEMITE CAMPSITE AVAILABILITY MONITOR")
        print("=" * 70)

        # Convert single dates to date ranges based on nights
        date_ranges = []
        for date_str in target_dates:
            start_date = datetime.strptime(date_str, "%Y-%m-%d")
            end_date = start_date + timedelta(days=nights)
            date_ranges.append((start_date, end_date))

        # Check each campground for each date range
        for campground_name in self.yosemite_campgrounds:
            for start_date, end_date in date_ranges:
                self.check_availability(campground_name, start_date, end_date)
                time.sleep(2)  # Be respectful between requests


def main():
    """Main function"""
    # Your RIDB API key
    RIDB_API_KEY = "da7bd758-b219-4a80-b885-556101d03afb"

    print("🏕️ YOSEMITE AVAILABILITY CHECK")
    print("Using the recgov library approach")
    print("=" * 50)

    # Install requirements reminder
    try:
        from fake_useragent import UserAgent
    except ImportError:
        print("❌ Missing requirement: pip install fake-useragent")
        return

    # Create monitor
    monitor = YosemiteAvailabilityMonitor(RIDB_API_KEY)

    # Example 1: Check specific date ranges
    print("🎯 OPTION 1: Check specific date ranges")
    date_ranges = [
        ("2025-06-09", "2025-06-10"),  # 2 nights: July 15-17
        ("2025-07-20", "2025-07-23"),  # 3 nights: July 20-23
    ]
    monitor.monitor_date_ranges(date_ranges)

    print("\n" + "=" * 70)
    print("🎯 OPTION 2: Check arrival dates with configurable nights")

    # Example 2: Check arrival dates with specified number of nights
    arrival_dates = ["2025-05-27", "2025-07-01", "2025-07-15"]
    nights = 2  # Stay for 2 nights

    print(f"Checking for {nights}-night stays starting on each date:")
    monitor.monitor_all_campgrounds(arrival_dates, nights=nights)

    print("\n" + "=" * 70)
    print("✅ AVAILABILITY CHECK COMPLETE")
    print("If sites are available, book immediately at the provided URLs!")
    print("=" * 70)


if __name__ == "__main__":
    main()

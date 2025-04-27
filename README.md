# OPNManager

OPNManager is a streamlined, user-friendly application designed to simplify the management of OPNsense firewalls. Built with Tauri and SvelteKit, this cross-platform app provides an intuitive interface for users who need a more simplified alternative to the standard OPNsense web interface.

## Network Topology Visualization Optimization (April 2025)

The network topology visualization feature has been significantly optimized to handle large networks and high-availability setups. Changes include:

### Frontend Optimizations:
- **Incremental rendering**: Added chunked rendering to avoid UI freezes with large networks
- **Dynamic device limiting**: Automatically adjusts visible device count based on total network size
- **Performance mode improvements**: Added better UI indication and automatic fallback for large networks
- **Reduced event overhead**: Simplified event dispatching system to reduce redundant operations
- **Timeout handling**: Added proper timeouts with fallbacks for data loading
- **User experience improvements**: Shows interfaces immediately while devices load incrementally
- **DOM element reduction**: Disabled node dragging and limited SVG element generation

### Backend Optimizations:
- **Parallel data fetching**: Using tokio's join for concurrent data retrieval
- **Improved timeout handling**: Added proper timeout handling with partial data returns
- **Memory pre-allocation**: Pre-allocate collections to avoid reallocation overhead
- **Selective sorting**: Only sort collections when they contain multiple items
- **Increased page size**: Reduced the number of API requests by increasing page size
- **Better progress reporting**: Added detailed logging to track data loading progress

These optimizations significantly improve the performance for users with complex network topologies, particularly in high-availability setups or networks with many devices.

## Disclaimer

**OPNManager is an independent project and is not affiliated with or endorsed by the OPNsense project or its developers.** This application is provided "as-is" without any warranties or guarantees. Users should exercise caution and ensure they understand the risks associated with granting API access

## Support the Project

If you find this application useful, consider supporting its development or downloading it from the official app stores.

<div align="center">
  <a href="https://apps.apple.com/us/app/opnmanager/id6743677680?platform=iphone" target="_blank">
    <img src="https://developer.apple.com/assets/elements/badges/download-on-the-app-store.svg" alt="Download on the App Store" height="40">
  </a>
  &nbsp;&nbsp;
  <a href="https://play.google.com/store/apps/details?id=com.OPNManager.app" target="_blank">
    <img src="https://upload.wikimedia.org/wikipedia/commons/7/78/Google_Play_Store_badge_EN.svg" alt="Get it on Google Play" height="40">
  </a>
  <br><br>
  <a href="https://www.buymeacoffee.com/swingline" target="_blank">
    <img src="https://cdn.buymeacoffee.com/buttons/default-orange.png" alt="Buy Me A Coffee" height="41" width="174">
  </a>
</div>



## Features

- **Updates**: Check for updates, read change log and update firewall.
- **Dashboard**: Get a quick overview of your OPNsense system's status, including gateway information and service status.
- **Device Management**: View devices on your network, including the ability to flush the ARP table.
- **Firewall Rules**: Enable and disable firewall rules created with the firewall automation plugin.
    - https://docs.opnsense.org/development/api/core/firewall.html
- **Alias Management**: Easily view, add, and remove IP addresses from firewall aliases.
- **Simple Settings**: Configure API access and update security PIN with a straightforward interface.
- **Multiple Firewall management**: Add multiple Firewall profiles and IP addresses
- **Dark Mode**: Add dark/light mode theme switch to top Navbar

## Purpose

This application is tailored for clients who require a simplified interface for managing their OPNsense firewall. It provides essential functionality for immediate actions while maintaining a balance between usability and security. For more complex configuration changes, users are encouraged to use the OPNsense webUI.

## Getting Started

1. Download the appropriate version for your operating system from the releases page.
2. Install the application on your device.
3. On first run, you'll be prompted to enter your OPNsense API credentials and create a security PIN.
4. Once set up, you can log in and start managing your OPNsense firewall.

## Security Note

**WARNING:** Never share your API keys with anyone.

This application uses API access to interact with your OPNsense firewall. Ensure that you're using a dedicated API key with appropriate permissions.

## Feedback and Contributions

Your feedback and contributions are welcome! Please open an issue or submit a pull request on our GitHub repository.



---

## Screenshot Gallery  

Below are screenshots showcasing OPNManager's interface and features:  

<div align="center">  
  <img src="./screenshots/phone/Screenshot_20250309_181351.png" alt="Screenshot 1" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250309_194754.png" alt="Screenshot 2" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250309_200250.png" alt="Screenshot 3" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250309_200308.png" alt="Screenshot 4" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250309_200320.png" alt="Screenshot 5" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250309_200331.png" alt="Screenshot 6" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250309_200348.png" alt="Screenshot 7" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250309_200359.png" alt="Screenshot 8" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250309_200413.png" alt="Screenshot 9" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250311_210558.png" alt="Screenshot 10" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250311_210622.png" alt="Screenshot 11" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250311_210716.png" alt="Screenshot 12" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250311_210739.png" alt="Screenshot 13" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250311_210757.png" alt="Screenshot 14" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250311_210815.png" alt="Screenshot 15" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250311_210834.png" alt="Screenshot 16" width="250px">  
  <img src="./screenshots/phone/Screenshot_20250311_211233.png" alt="Screenshot 17" width="250px">  
</div>  

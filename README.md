# OPNManager

OPNManager is a streamlined, user-friendly application designed to simplify the management of OPNsense firewalls. Built with Tauri and SvelteKit, this cross-platform app provides an intuitive interface for users who need a more simplified alternative to the standard OPNsense web interface.

## Disclaimer

**OPNManager is an independent project and is not affiliated with or endorsed by the OPNsense project or its developers.** This application is provided "as-is" without any warranties or guarantees. Users should exercise caution and ensure they understand the risks associated with granting API access to their firewall.

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

This application uses API access to interact with your OPNsense firewall. Ensure that you're using a dedicated API key with appropriate permissions for security best practices.

Recommended minimum API permissions to use the current version of the app:

| Type | Name                        |
|------|-----------------------------|
| GUI  | Dashboard (all)   |
| GUI  | Diagnostics: ARP Table      |
| GUI  | Diagnostics: Reboot System  |
| GUI  | Firewall: Alias: Edit       |
| GUI  | Firewall: Automation: Filter|
| GUI  | Status: Services            |

## Support the Project

If you find this application useful, consider supporting its development:

<div style="text-align: center;">
    <a href="https://www.buymeacoffee.com/swingline" target="_blank">
        <img src="https://cdn.buymeacoffee.com/buttons/default-orange.png" alt="Buy Me A Coffee" height="41" width="174">
    </a>
</div>

## Feedback and Contributions

Your feedback and contributions are welcome! Please open an issue or submit a pull request on our GitHub repository.

## Screenshots

|  ![Screenshot 1](https://github.com/user-attachments/assets/4a3f55d2-88e1-4ab5-870e-3c843659129b)  | ![96172b51-1762-4b01-926d-0663e7b35348](https://github.com/user-attachments/assets/0be2170a-e466-4bc2-956d-3ff48c594e2f) | ![596b5647-a6b7-46ba-8f31-e4e664b03ff0](https://github.com/user-attachments/assets/401cd759-f28e-4ba6-8a60-e4a2289ceeb1) |
|---|---|---|
| ![804ef38c-2c80-4b54-8f69-8b0013e88bcc](https://github.com/user-attachments/assets/1be9b256-fd8c-47d1-8a55-3cceee66ca5f) | ![Screenshot 3](https://github.com/user-attachments/assets/c7129725-b9cc-400c-bfdf-c56df1a28284)  |
| ![Screenshot 4](https://github.com/user-attachments/assets/36f17388-6053-4b43-9dda-5221ba607b5b) | ![7615fe31-577d-429e-8236-72465b05234b](https://github.com/user-attachments/assets/6bf5e237-1e56-4921-ac67-488a04220ae2) | ![Screenshot 6](https://github.com/user-attachments/assets/358dab68-280c-48de-bd33-0ed9b5a02643) |
| ![Screenshot 7](https://github.com/user-attachments/assets/fa9dfa64-b79d-46bb-962b-d91a2f12dfff) | ![Screenshot 8](https://github.com/user-attachments/assets/a6045ee6-7bc3-4062-9e40-1a78d55ed900) | ![Screenshot 9](https://github.com/user-attachments/assets/abfa9c49-daed-4458-9abe-9d5a23b2a328) |
| ![Screenshot 10](https://github.com/user-attachments/assets/9b0bf147-6883-4c5a-9891-79fded253762) | ![Screenshot 11](https://github.com/user-attachments/assets/4330a3cd-ab23-40a4-93fb-d5e7c08c98dd) | |



apk sign 
/home/steve/Android/Sdk/build-tools/34.0.0/apksigner sign --ks my-release-key.jks --out OPNManager.apk /home/steve/OPNManager/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk
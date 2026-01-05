This repository is destinated to provide only the frontend interface for tutorlolv2 project, the Tauri handler and events will stay in tutorlolv2_desktop_app repository, but all the constructed components at tutorlolv2_frontend will be moved to here

There might be added custom features to the main crate which will determine if:
- Only the overlay should be compiled
- If documentation and hover effects should be included (will make the final application slightly lighter and faster, while consuming less memory)
- Include only the resources that will be available in the web environment
- All features included

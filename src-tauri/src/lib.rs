mod db;
mod commands;
mod http_client;
mod devices;
mod alias;
mod dashboard;
mod firewall;
mod power;
mod traffic;
mod update_checker;
mod firewall_logs;
mod routes; 
mod system_resources;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            let db = Database::new(app.handle()).expect("Failed to initialize database");
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_first_run, 
            commands::save_initial_config, 
            commands::verify_pin, 
            commands::get_api_info, 
            commands::update_api_info,
            commands::get_api_profiles,
            commands::update_pin,
            commands::get_vendor_info,
            commands::add_api_profile,
            commands::delete_api_profile,
            commands::set_default_profile,
            devices::get_devices,
            devices::flush_arp_table,
            alias::list_network_aliases,
            alias::remove_ip_from_alias,
            alias::add_ip_to_alias,
            alias::get_alias,
            alias::search_alias_items,
            alias::toggle_alias,
            alias::delete_alias,
            alias::apply_alias_changes,
            alias::add_alias,
            dashboard::get_gateway_status,  
            dashboard::get_services,    
            dashboard::restart_service,
            firewall::get_firewall_rules,
            firewall::toggle_firewall_rule,  
            firewall::apply_firewall_changes,
            firewall_logs::get_log_filters,
            firewall_logs::get_interface_names,
            firewall_logs::get_firewall_logs,
            firewall_logs::apply_filters,
            firewall_logs::limit_logs,
            routes::get_routes,
            routes::get_route_info,
            routes::add_route,
            routes::delete_route,
            routes::toggle_route,
            routes::apply_changes,
            power::reboot_firewall,
            traffic::get_interface_traffic,
            update_checker::get_current_firmware_status,
            update_checker::check_for_updates,
            update_checker::get_changelog,
            update_checker::start_update,
            system_resources::get_system_resources,
            system_resources::get_system_disk,
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
mod alias;
mod commands;
mod dashboard;
mod db;
mod devices;
mod firewall;
mod firewall_logs;
mod http_client;
mod pin_cache;
mod power;
mod routes;
mod snapshots;
mod system_resources;
mod traffic;
mod unbound;
mod update_checker;

use db::Database;
use firewall_logs::register_log_cache;
use pin_cache::PinCache;
use tauri::Manager;
use traffic::register_traffic_cache;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            let pin_cache = PinCache::new();
            app.manage(pin_cache);

            let db = Database::new(app.handle()).expect("Failed to initialize database");
            app.manage(db);

            register_log_cache(app).expect("Failed to register log cache");
            register_traffic_cache(app).expect("Failed to register traffic cache");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_first_run,
            commands::save_initial_config,
            commands::get_api_info,
            commands::update_api_info,
            commands::get_api_profiles,
            commands::update_pin,
            commands::get_vendor_info,
            commands::add_api_profile,
            commands::delete_api_profile,
            commands::set_default_profile,
            commands::test_api_connection,
            commands::get_dashboard_preferences,
            commands::save_dashboard_preferences,
            pin_cache::set_pin,
            pin_cache::clear_pin,
            pin_cache::verify_pin,
            devices::get_devices,
            devices::get_ndp_devices,
            devices::get_combined_devices,
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
            dashboard::get_system_time,
            firewall::get_firewall_rules,
            firewall::toggle_firewall_rule,
            firewall::apply_firewall_changes,
            firewall::get_rule_template,
            firewall::add_firewall_rule,
            firewall::delete_firewall_rule,
            firewall::list_network_select_options,
            firewall::set_rule,
            firewall::get_rule,
            firewall_logs::get_log_filters,
            firewall_logs::get_interface_names,
            firewall_logs::get_firewall_logs,
            firewall_logs::update_log_filters,
            firewall_logs::start_log_polling,
            firewall_logs::stop_log_polling,
            firewall_logs::clear_log_cache,
            routes::get_routes,
            routes::get_route_info,
            routes::add_route,
            routes::delete_route,
            routes::toggle_route,
            routes::apply_changes,
            power::reboot_firewall,
            snapshots::is_snapshots_supported,
            snapshots::get_snapshots,
            snapshots::get_new_snapshot,
            snapshots::get_snapshot,
            snapshots::add_snapshot,
            snapshots::delete_snapshot,
            snapshots::activate_snapshot,
            snapshots::update_snapshot,
            traffic::get_interface_traffic,
            traffic::get_traffic_graph_data,
            traffic::update_traffic_data,
            traffic::clear_traffic_cache,
            update_checker::get_current_firmware_status,
            update_checker::check_for_updates,
            update_checker::get_changelog,
            update_checker::start_update,
            system_resources::get_system_resources,
            system_resources::get_system_disk,
            system_resources::get_system_temperature,
            unbound::get_unbound_settings,
            unbound::set_dnsbl_settings,
            unbound::apply_dnsbl_settings,
            unbound::get_dnsbl_cron_job,
            unbound::add_dnsbl_cron_job,
            unbound::delete_dnsbl_cron_job,
            unbound::apply_cron_changes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

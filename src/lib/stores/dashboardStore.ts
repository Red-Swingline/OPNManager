import { writable } from 'svelte/store';
import { invoke } from "@tauri-apps/api/core";

export interface DashboardWidgetPref {
  widget_key: string;
  visible: boolean;
  position: number;
}

const DEFAULT_WIDGETS: DashboardWidgetPref[] = [
  { widget_key: 'uptime', visible: true, position: 0 },
  { widget_key: 'memory', visible: true, position: 1 },
  { widget_key: 'cpu_temp', visible: true, position: 2 },
  { widget_key: 'disk', visible: true, position: 3 },
  { widget_key: 'services', visible: true, position: 4 },
  { widget_key: 'traffic_graph', visible: true, position: 5 },
  { widget_key: 'gateways', visible: true, position: 6 },
  { widget_key: 'interfaces', visible: true, position: 7 },
  { widget_key: 'wol', visible: true, position: 8 },
];

function createDashboardStore() {
  const { subscribe, set, update } = writable<{
    widgets: DashboardWidgetPref[];
    isLoaded: boolean;
    isEditing: boolean;
  }>({
    widgets: [...DEFAULT_WIDGETS],
    isLoaded: false,
    isEditing: false
  });

  return {
    subscribe,
    loadPreferences: async () => {
      try {
        const prefs = await invoke<Record<string, DashboardWidgetPref>>('get_dashboard_preferences');

        if (prefs && Object.keys(prefs).length > 0) {
          update(state => ({
            ...state,
            widgets: Object.values(prefs).sort((a, b) => a.position - b.position),
            isLoaded: true
          }));
        } else {
          update(state => ({
            ...state,
            widgets: [...DEFAULT_WIDGETS],
            isLoaded: true
          }));
        }
      } catch (error) {
        console.error('Failed to load dashboard preferences:', error);
        update(state => ({
          ...state,
          widgets: [...DEFAULT_WIDGETS],
          isLoaded: true
        }));
      }
    },

    savePreferences: async () => {
      try {
        let state;
        update(s => {
          state = s;
          return s;
        });

        if (state.widgets) {
          await invoke('save_dashboard_preferences', { prefs: state.widgets });
          return true;
        }
        return false;
      } catch (error) {
        console.error('Failed to save dashboard preferences:', error);
        return false;
      }
    },

    toggleWidget: (key: string) => {
      update(state => {
        const widgets = state.widgets.map(widget => {
          if (widget.widget_key === key) {
            return { ...widget, visible: !widget.visible };
          }
          return widget;
        });

        return { ...state, widgets };
      });
    },

    updatePositions: (orderedKeys: string[]) => {
      update(state => {
        const keyToWidget = new Map(state.widgets.map(w => [w.widget_key, w]));
        const widgets = orderedKeys.map((key, index) => {
          const widget = keyToWidget.get(key);
          if (widget) {
            return { ...widget, position: index };
          }
          return null;
        }).filter(Boolean) as DashboardWidgetPref[];

        return { ...state, widgets };
      });
    },

    resetToDefaults: () => {
      update(state => ({
        ...state,
        widgets: [...DEFAULT_WIDGETS]
      }));
    },

    toggleEditMode: () => {
      update(state => ({
        ...state,
        isEditing: !state.isEditing
      }));
    },

    exitEditMode: () => {
      update(state => ({
        ...state,
        isEditing: false
      }));
    }
  };
}

export const dashboardStore = createDashboardStore();
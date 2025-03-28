import { writable } from 'svelte/store';
import { invoke } from "@tauri-apps/api/core";

interface AuthState {
  isLoggedIn: boolean;
  isConfigured: boolean;
}

async function cleanupOnLogout() {
  console.log("Performing logout cleanup");
  
  try {

    await invoke("clear_traffic_cache").catch(err => {
      console.error("Error clearing traffic cache:", err);
    });

    await invoke("stop_log_polling").catch(err => {
      console.error("Error stopping log polling:", err);
    });

    await invoke("clear_pin").catch(err => {
      console.error("Error clearing PIN cache:", err);
    });
    
    console.log("Logout cleanup completed");
    return true;
  } catch (error) {
    console.error("Error during logout cleanup:", error);
    return false;
  }
}

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>({
    isLoggedIn: false,
    isConfigured: false
  });

  return {
    subscribe,
    login: () => update(state => ({ ...state, isLoggedIn: true })),
    logout: async () => {
      await cleanupOnLogout();
      update(state => ({ ...state, isLoggedIn: false }));
    },
    setConfigured: (value: boolean) => update(state => ({ ...state, isConfigured: value })),
    reset: () => set({ isLoggedIn: false, isConfigured: false })
  };
}

export const authStore = createAuthStore();
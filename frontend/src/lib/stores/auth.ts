import { writable } from "svelte/store";
import { browser } from "$app/environment";

export interface User {
  id: string;
  email: string;
  name: string | null;
  role: string;
}

export interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
}

const getInitialState = (): AuthState => {
  if (browser) {
    const token = localStorage.getItem("token");
    const userStr = localStorage.getItem("user");
    if (token && userStr) {
      try {
        const user = JSON.parse(userStr);
        return { user, token, isAuthenticated: true };
      } catch {
        localStorage.removeItem("token");
        localStorage.removeItem("user");
      }
    }
  }
  return { user: null, token: null, isAuthenticated: false };
};

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>(getInitialState());

  return {
    subscribe,
    login: (token: string, user: User) => {
      if (browser) {
        localStorage.setItem("token", token);
        localStorage.setItem("user", JSON.stringify(user));
      }
      set({ user, token, isAuthenticated: true });
    },
    logout: () => {
      if (browser) {
        localStorage.removeItem("token");
        localStorage.removeItem("user");
      }
      set({ user: null, token: null, isAuthenticated: false });
    },
    updateUser: (user: User) => {
      if (browser) {
        localStorage.setItem("user", JSON.stringify(user));
      }
      update((state) => ({ ...state, user }));
    },
  };
}

export const auth = createAuthStore();

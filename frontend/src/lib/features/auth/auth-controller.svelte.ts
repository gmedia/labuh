import { goto } from "$app/navigation";
import { api } from "$lib/api";

export class AuthController {
  name = $state("");
  email = $state("");
  password = $state("");
  confirmPassword = $state("");
  error = $state("");
  loading = $state(false);

  async login(e: Event) {
    e.preventDefault();
    this.error = "";
    this.loading = true;

    const result = await api.auth.login({
      email: this.email,
      password: this.password,
    });

    if (result.error) {
      this.error = result.message || "Login failed";
      this.loading = false;
      return;
    }

    goto("/dashboard");
  }

  async register(e: Event) {
    e.preventDefault();
    this.error = "";

    if (this.password !== this.confirmPassword) {
      this.error = "Passwords do not match";
      return;
    }

    if (this.password.length < 6) {
      this.error = "Password must be at least 6 characters";
      return;
    }

    this.loading = true;

    const result = await api.auth.register({
      email: this.email,
      password: this.password,
      name: this.name || undefined,
    });

    if (result.error) {
      this.error = result.message || "Registration failed";
      this.loading = false;
      return;
    }

    goto("/dashboard");
  }
}

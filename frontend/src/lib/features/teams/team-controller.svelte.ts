import {
  api,
  type TeamResponse,
  type TeamMember,
  type TeamRole,
} from "$lib/api";
import { activeTeam } from "$lib/stores";
import { toast } from "svelte-sonner";
import { get } from "svelte/store";

export class TeamController {
  teams = $state<TeamResponse[]>([]);
  loadingTeams = $state(true);
  newTeamName = $state("");
  creatingTeam = $state(false);

  selectedTeamMembers = $state<TeamMember[]>([]);
  loadingMembers = $state(false);
  inviteEmail = $state("");
  inviteRole = $state<TeamRole>("Developer");
  inviting = $state(false);

  readonly roles: TeamRole[] = ["Owner", "Admin", "Developer", "Viewer"];

  async init() {
    await this.loadTeams();
  }

  async loadTeams() {
    this.loadingTeams = true;
    const result = await api.teams.list();
    if (result.data) {
      this.teams = result.data;
      if (!get(activeTeam) && this.teams.length > 0) {
        activeTeam.setActiveTeam(this.teams[0]);
      }
    }
    this.loadingTeams = false;
  }

  async createTeam() {
    if (!this.newTeamName) return;
    this.creatingTeam = true;
    const result = await api.teams.create(this.newTeamName);
    if (result.data) {
      toast.success(`Team "${this.newTeamName}" created`);
      this.newTeamName = "";
      await this.loadTeams();
    } else {
      toast.error(result.message || result.error || "Failed to create team");
    }
    this.creatingTeam = false;
  }

  async deleteTeam(id: string) {
    if (
      !confirm(
        "Are you sure you want to delete this team? This action cannot be undone and will delete all stacks and resources within the team.",
      )
    )
      return;

    const result = await api.teams.remove(id);
    if (!result.error) {
      toast.success("Team deleted successfully");
      // If the deleted team was active, switch to another one or clear active
      const current = get(activeTeam);
      if (current && current.team.id === id) {
        activeTeam.setActiveTeam(null);
      }
      await this.loadTeams();
      // Reload page to ensure clean state if needed, or let router handle it
      if (!get(activeTeam) && this.teams.length > 0) {
        activeTeam.setActiveTeam(this.teams[0]);
      } else if (this.teams.length === 0) {
        // No teams left?
      }
    } else {
      toast.error(result.message || result.error || "Failed to delete team");
    }
  }

  async loadMembers(teamId: string) {
    this.loadingMembers = true;
    const result = await api.teams.listMembers(teamId);
    if (result.data) {
      this.selectedTeamMembers = result.data;
    }
    this.loadingMembers = false;
  }

  async inviteMember(teamId: string) {
    if (!this.inviteEmail) return;
    this.inviting = true;
    const result = await api.teams.addMember(
      teamId,
      this.inviteEmail,
      this.inviteRole,
    );
    if (!result.error) {
      toast.success(`Invited ${this.inviteEmail}`);
      this.inviteEmail = "";
      await this.loadMembers(teamId);
    } else {
      toast.error(result.message || result.error);
    }
    this.inviting = false;
  }

  async removeMember(teamId: string, userId: string) {
    if (!confirm("Are you sure you want to remove this member?")) return;
    const result = await api.teams.removeMember(teamId, userId);
    if (!result.error) {
      toast.success("Member removed");
      await this.loadMembers(teamId);
    } else {
      toast.error(result.message || result.error);
    }
  }

  async updateRole(teamId: string, userId: string, role: TeamRole) {
    const result = await api.teams.updateMemberRole(teamId, userId, role);
    if (!result.error) {
      toast.success("Role updated");
      await this.loadMembers(teamId);
    } else {
      toast.error(result.message || result.error);
    }
  }
}

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
  inviteName = $state("");
  inviteEmail = $state("");
  invitePassword = $state("");
  inviteRole = $state<TeamRole>("Developer");
  inviting = $state(false);

  // UI States (Modals)
  showDeleteTeamConfirm = $state(false);
  teamToDelete = $state<string | null>(null);
  showRemoveMemberConfirm = $state(false);
  memberToRemove = $state<{ teamId: string; userId: string } | null>(null);

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

  requestDeleteTeam(id: string) {
    this.teamToDelete = id;
    this.showDeleteTeamConfirm = true;
  }

  async confirmDeleteTeam() {
    if (!this.teamToDelete) return;
    const id = this.teamToDelete;
    this.showDeleteTeamConfirm = false;

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
    const result = await api.teams.getMembers(teamId);
    if (result.data) {
      this.selectedTeamMembers = result.data;
    }
    this.loadingMembers = false;
  }

  async inviteMember(teamId: string) {
    if (!this.inviteEmail || !this.inviteName || !this.invitePassword) {
      toast.error("Please fill in all fields (Name, Email, Password)");
      return;
    }
    this.inviting = true;
    const result = await api.teams.addMember(teamId, {
      name: this.inviteName,
      email: this.inviteEmail,
      password: this.invitePassword,
      role: this.inviteRole,
    });
    if (!result.error) {
      toast.success(`Invited ${this.inviteEmail}`);
      this.inviteName = "";
      this.inviteEmail = "";
      this.invitePassword = "";
      await this.loadMembers(teamId);
    } else {
      toast.error(result.message || result.error);
    }
    this.inviting = false;
  }

  requestRemoveMember(teamId: string, userId: string) {
    this.memberToRemove = { teamId, userId };
    this.showRemoveMemberConfirm = true;
  }

  async confirmRemoveMember() {
    if (!this.memberToRemove) return;
    const { teamId, userId } = this.memberToRemove;
    this.showRemoveMemberConfirm = false;

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

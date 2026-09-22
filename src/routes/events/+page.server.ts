import { loadLocations } from '$lib/server/load-admin';
import { loadTeam } from '$lib/server/load-team';

export const load = async () => {
  const [team, locations] = await Promise.all([loadTeam(), loadLocations()]);

  return {
    events: team.events,
    locations,
    players: team.players,
    season: team.season,
    teamName: team.teamName,
  };
};

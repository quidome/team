import { error } from '@sveltejs/kit';

import { loadTeam } from '$lib/server/load-team';

export const load = async ({ params }) => {
  const team = await loadTeam();
  const event = team.events.find(
    (candidate) =>
      candidate.id === params.eventId &&
      (candidate.type === 'training' ||
        (candidate.status === 'scheduled' &&
          (candidate.homeTeamName === team.teamName || candidate.awayTeamName === team.teamName))),
  );

  if (!event) {
    error(404, 'Event not found');
  }

  return {
    event,
    players: team.players,
    teamName: team.teamName,
  };
};

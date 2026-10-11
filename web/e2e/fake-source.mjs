// A fake game list source for the browser tests, so they never call the real source and do not
// depend on the date. It serves the real answers from crates/server/tests/data/game-list/:
// - /db/json/club.pl: the club list;
// - /db/json/wedstrijd.pl?clb_ID=37: Blue Drakes' games, with `seizoen` set to the asked season;
// - any other club: a season with no games.
import { readFileSync } from 'node:fs';
import { createServer } from 'node:http';

const port = Number(process.env.FAKE_SOURCE_PORT ?? 4102);
const dataDir = new URL('../../crates/server/tests/data/game-list/', import.meta.url);
const read = (name) => JSON.parse(readFileSync(new URL(name, dataDir), 'utf8'));

const clubs = read('clubs.json');
const drakesGames = read('games-37-2026-2027.json');
const emptySeason = read('games-empty.json');

function games(club, season) {
	const source = club === '37' ? drakesGames : emptySeason;
	return {
		...source,
		seizoen: season,
		wedstrijden: source.wedstrijden.map((game) => ({ ...game, seizoen: season }))
	};
}

createServer((request, response) => {
	const url = new URL(request.url ?? '/', `http://localhost:${port}`);
	let body;
	if (url.pathname === '/health') body = { ok: true };
	else if (url.pathname === '/db/json/club.pl') body = clubs;
	else if (url.pathname === '/db/json/wedstrijd.pl') {
		body = games(url.searchParams.get('clb_ID'), url.searchParams.get('seizoen'));
	}
	if (body === undefined) {
		response.writeHead(404).end();
		return;
	}
	response.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify(body));
}).listen(port, '127.0.0.1');

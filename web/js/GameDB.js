export async function initDB() {
    // Server-side DB — nothing to init on client
}

export async function insertGame({ boardSize, blackPlayer, whitePlayer, result, blackScore, whiteScore, sgf, playedAt }) {
    try {
        const res = await fetch('/api/games', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ boardSize, blackPlayer, whitePlayer, result, blackScore, whiteScore, sgf, playedAt }),
        });
        const data = await res.json();
        return data.id;
    } catch (e) {
        console.error('Failed to save game:', e);
        return null;
    }
}

export async function getRecentGames(limit = 100) {
    try {
        const res = await fetch(`/api/games?limit=${limit}`);
        return await res.json();
    } catch (e) {
        console.error('Failed to load games:', e);
        return [];
    }
}

export async function getGameSGF(id) {
    try {
        const res = await fetch(`/api/games/${id}/sgf`);
        const data = await res.json();
        return data.sgf;
    } catch (e) {
        console.error('Failed to load SGF:', e);
        return null;
    }
}

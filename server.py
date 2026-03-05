#!/usr/bin/env python3

import http.server
import json
import os
import sqlite3
from urllib.parse import urlparse, parse_qs


ROOT_DIR = os.path.dirname(os.path.abspath(__file__))
DB_DIR = os.path.join(ROOT_DIR, 'data')
DB_PATH = os.path.join(DB_DIR, 'othello.db')
WEB_DIR = os.path.join(ROOT_DIR, 'web')
PKG_DIR = os.path.join(ROOT_DIR, 'pkg')
PORT = 8080


def init_db():
    os.makedirs(DB_DIR, exist_ok=True)
    conn = sqlite3.connect(DB_PATH)
    conn.execute('''
        CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            board_size INTEGER NOT NULL,
            black_player TEXT NOT NULL,
            white_player TEXT NOT NULL,
            result TEXT NOT NULL,
            black_score INTEGER NOT NULL,
            white_score INTEGER NOT NULL,
            sgf TEXT NOT NULL,
            played_at TEXT NOT NULL
        )
    ''')
    conn.commit()
    conn.close()


def get_db():
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    return conn


class OthelloHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=WEB_DIR, **kwargs)

    def translate_path(self, path):
        """Serve /pkg/ from project root pkg/ dir, everything else from web/."""
        parsed = urlparse(path)
        if parsed.path.startswith('/pkg/'):
            rel = parsed.path[len('/pkg/'):]
            return os.path.join(PKG_DIR, rel)
        return super().translate_path(path)

    def do_GET(self):
        parsed = urlparse(self.path)

        if parsed.path == '/api/games':
            self.handle_list_games(parsed)
        elif parsed.path.startswith('/api/games/') and parsed.path.endswith('/sgf'):
            game_id = parsed.path.split('/')[3]
            self.handle_get_sgf(game_id)
        else:
            super().do_GET()

    def do_POST(self):
        parsed = urlparse(self.path)

        if parsed.path == '/api/games':
            self.handle_insert_game()
        else:
            self.send_error(404)

    def handle_list_games(self, parsed):
        qs = parse_qs(parsed.query)
        limit = int(qs.get('limit', [100])[0])

        conn = get_db()
        rows = conn.execute(
            'SELECT id, board_size, black_player, white_player, result, '
            'black_score, white_score, played_at '
            'FROM games ORDER BY id DESC LIMIT ?',
            (limit,)
        ).fetchall()
        conn.close()

        games = [dict(r) for r in rows]
        self.send_json(games)

    def handle_get_sgf(self, game_id):
        conn = get_db()
        row = conn.execute('SELECT sgf FROM games WHERE id = ?', (game_id,)).fetchone()
        conn.close()

        if row:
            self.send_json({'sgf': row['sgf']})
        else:
            self.send_error(404, 'Game not found')

    def handle_insert_game(self):
        length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(length)
        data = json.loads(body)

        conn = get_db()
        cur = conn.execute(
            'INSERT INTO games (board_size, black_player, white_player, result, '
            'black_score, white_score, sgf, played_at) '
            'VALUES (?, ?, ?, ?, ?, ?, ?, ?)',
            (
                data['boardSize'],
                data['blackPlayer'],
                data['whitePlayer'],
                data['result'],
                data['blackScore'],
                data['whiteScore'],
                data['sgf'],
                data['playedAt'],
            )
        )
        conn.commit()
        game_id = cur.lastrowid
        conn.close()

        self.send_json({'id': game_id}, status=201)

    def send_json(self, data, status=200):
        body = json.dumps(data).encode()
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', len(body))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format, *args):
        # Only log API calls and errors
        if '/api/' in (args[0] if args else ''):
            super().log_message(format, *args)


if __name__ == '__main__':
    init_db()
    print(f'Othello server running at http://localhost:{PORT}')
    print(f'Database: {DB_PATH}')
    server = http.server.HTTPServer(('', PORT), OthelloHandler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print('\nShutting down.')
        server.server_close()

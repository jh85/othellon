import { getWasm } from '../OthelloWasm.js';
import { insertGame, getRecentGames, getGameSGF } from '../GameDB.js';

const BOARD_COLOR = 0x2d6a4f;
const GRID_COLOR = 0x1b4332;
const BLACK_COLOR = 0x111111;
const WHITE_COLOR = 0xeeeeee;
const HINT_BLACK = 0x114411;
const HINT_WHITE = 0xcccccc;
const PANEL_W = 180;
const RIGHT_W = 200;
const PANEL_COLOR = 0x16213e;
const PANEL_ACCENT = 0x2a3a5c;
const MAX_CELL = 60;
const GRAPH_HEIGHT = 80;

function playerLabel(type) {
    if (type === 'human') return 'Human';
    return `AI-L${type.slice(2)}`;
}

function sgfCoord(row, col) {
    return String.fromCharCode(97 + col) + String.fromCharCode(97 + row);
}

export class GameScene extends Phaser.Scene {
    constructor() {
        super({ key: 'GameScene' });
    }

    create() {
        this.selectedSize = 8;
        this.selectedBlack = 'human';
        this.selectedWhite = 'human';

        this.piecesGroup = this.add.group();
        this.hintsGroup = this.add.group();
        this.graphLines = null;
        this.boardGfx = null;
        this.boardLabels = [];
        this.graphBg = null;
        this.graphLabels = [];

        this.logEntries = [];

        this.drawPanel();
        this.drawRightPanel();
        this.refreshLog();
        this.startGame();
    }

    /* ─── Left Panel ─── */

    drawPanel() {
        const H = this.cameras.main.height;

        const g = this.add.graphics();
        g.fillStyle(PANEL_COLOR);
        g.fillRect(0, 0, PANEL_W, H);
        g.lineStyle(1, PANEL_ACCENT);
        g.lineBetween(PANEL_W, 0, PANEL_W, H);

        const cx = PANEL_W / 2;

        this.add.text(cx, 24, 'OTHELLO', {
            fontSize: '24px', fontFamily: 'monospace', color: '#e0e0e0',
        }).setOrigin(0.5);

        this.add.text(cx, 64, 'Board Size', {
            fontSize: '14px', fontFamily: 'monospace', color: '#8899aa',
        }).setOrigin(0.5);

        const sizes = [4, 6, 8, 10, 12, 14, 16, 18, 20];
        this.sizeButtons = [];
        const cols = 3;
        const btnW = 42;
        const btnH = 24;
        const gapX = 6;
        const gapY = 4;
        const gridW = cols * btnW + (cols - 1) * gapX;
        const startX = cx - gridW / 2 + btnW / 2;
        let sy = 88;

        sizes.forEach((s, i) => {
            const col = i % cols;
            const row = Math.floor(i / cols);
            const x = startX + col * (btnW + gapX);
            const y = sy + row * (btnH + gapY);
            const btn = this.add.text(x, y, `${s}`, {
                fontSize: '14px', fontFamily: 'monospace', color: '#888',
                backgroundColor: '#333', padding: { x: 8, y: 3 },
            }).setOrigin(0.5).setInteractive({ useHandCursor: true });
            btn.on('pointerdown', () => {
                this.selectedSize = s;
                this.updateSizeHighlight();
            });
            this.sizeButtons.push({ btn, size: s });
        });
        this.updateSizeHighlight();

        const blackY = sy + Math.ceil(sizes.length / cols) * (btnH + gapY) + 16;
        this.blackButtons = this.createPlayerSelector(cx, blackY, 'Black');

        const whiteY = blackY + 110;
        this.whiteButtons = this.createPlayerSelector(cx, whiteY, 'White');

        const startBtnY = whiteY + 120;
        const startBtn = this.add.text(cx, startBtnY, 'START', {
            fontSize: '18px', fontFamily: 'monospace', color: '#1a1a2e',
            backgroundColor: '#4ecca3', padding: { x: 20, y: 8 },
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });
        startBtn.on('pointerdown', () => this.startGame());
        startBtn.on('pointerover', () => startBtn.setStyle({ backgroundColor: '#6ee6be' }));
        startBtn.on('pointerout', () => startBtn.setStyle({ backgroundColor: '#4ecca3' }));

        this.autoContinue = false;
        this.autoBtn = this.add.text(cx, startBtnY + 40, 'AUTO: OFF', {
            fontSize: '13px', fontFamily: 'monospace', color: '#888',
            backgroundColor: '#333', padding: { x: 12, y: 5 },
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });
        this.autoBtn.on('pointerdown', () => this.toggleAuto());
    }

    createPlayerSelector(cx, y, label) {
        this.add.text(cx, y, label, {
            fontSize: '14px', fontFamily: 'monospace', color: '#8899aa',
        }).setOrigin(0.5);

        const side = label.toLowerCase();
        const selected = side === 'black' ? this.selectedBlack : this.selectedWhite;
        const buttons = [];

        const humanBtn = this.add.text(cx, y + 22, 'Human', {
            fontSize: '14px', fontFamily: 'monospace', color: '#888',
            backgroundColor: '#333', padding: { x: 12, y: 4 },
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });
        humanBtn.on('pointerdown', () => {
            if (side === 'black') this.selectedBlack = 'human';
            else this.selectedWhite = 'human';
            this.updatePlayerHighlight(buttons, 'human');
        });
        buttons.push({ btn: humanBtn, key: 'human' });

        const row1 = [0, 1, 2];
        const row2 = [3, 4];
        const lvlGap = 36;
        const row1W = row1.length * lvlGap;
        const aiLabelX = cx - row1W / 2 - 12;
        this.add.text(aiLabelX, y + 52, 'AI', {
            fontSize: '14px', fontFamily: 'monospace', color: '#8899aa',
        }).setOrigin(0.5);

        const lvlStartX = cx - row1W / 2 + lvlGap / 2;
        const makeLvlBtn = (lvl, x, btnY) => {
            const key = `ai${lvl}`;
            const btn = this.add.text(x, btnY, `L${lvl}`, {
                fontSize: '13px', fontFamily: 'monospace', color: '#888',
                backgroundColor: '#333', padding: { x: 5, y: 3 },
            }).setOrigin(0.5).setInteractive({ useHandCursor: true });
            btn.on('pointerdown', () => {
                if (side === 'black') this.selectedBlack = key;
                else this.selectedWhite = key;
                this.updatePlayerHighlight(buttons, key);
            });
            buttons.push({ btn, key });
        };

        row1.forEach((lvl, i) => makeLvlBtn(lvl, lvlStartX + i * lvlGap, y + 52));
        row2.forEach((lvl, i) => makeLvlBtn(lvl, lvlStartX + i * lvlGap, y + 78));

        this.updatePlayerHighlight(buttons, selected);
        return buttons;
    }

    updateSizeHighlight() {
        this.sizeButtons.forEach(({ btn, size }) => {
            btn.setStyle({
                color: size === this.selectedSize ? '#4ecca3' : '#888',
                backgroundColor: size === this.selectedSize ? '#2a4a3e' : '#333',
            });
        });
    }

    updatePlayerHighlight(buttons, selected) {
        buttons.forEach(({ btn, key }) => {
            btn.setStyle({
                color: key === selected ? '#4ecca3' : '#888',
                backgroundColor: key === selected ? '#2a4a3e' : '#333',
            });
        });
    }

    toggleAuto() {
        this.autoContinue = !this.autoContinue;
        this.autoBtn.setText(this.autoContinue ? 'AUTO: ON' : 'AUTO: OFF');
        this.autoBtn.setStyle({
            color: this.autoContinue ? '#4ecca3' : '#888',
            backgroundColor: this.autoContinue ? '#2a4a3e' : '#333',
        });
    }

    /* ─── Right Panel ─── */

    drawRightPanel() {
        const W = this.cameras.main.width;
        const H = this.cameras.main.height;
        this.rightX = W - RIGHT_W;

        const g = this.add.graphics();
        g.fillStyle(PANEL_COLOR);
        g.fillRect(this.rightX, 0, RIGHT_W, H);
        g.lineStyle(1, PANEL_ACCENT);
        g.lineBetween(this.rightX, 0, this.rightX, H);

        this.add.text(this.rightX + RIGHT_W / 2, 16, 'Recent Games', {
            fontSize: '14px', fontFamily: 'monospace', color: '#8899aa',
        }).setOrigin(0.5);

        // Scrollable area parameters
        this.logTopY = 38;
        this.logLineH = 20;
        this.logMaxVisible = Math.floor((H - this.logTopY - 10) / this.logLineH);
    }

    async refreshLog() {
        this.logEntries.forEach(e => e.destroy());
        this.logEntries = [];

        const games = await getRecentGames(this.logMaxVisible);
        // API returns newest first; reverse so oldest on top, newest at bottom
        games.reverse();

        games.forEach((game, i) => {
            const y = this.logTopY + i * this.logLineH;

            const bp = game.black_player || '';
            const wp = game.white_player || '';
            const bs = game.board_size || 8;
            const res = game.result || '';

            const bShort = bp === 'Human' ? 'Hum' : bp.replace('AI-', '');
            const wShort = wp === 'Human' ? 'Hum' : wp.replace('AI-', '');
            const summary = `${bs} ${bShort}v${wShort} ${res}`;

            const resultColor = res.startsWith('B') ? '#999' :
                res.startsWith('W') ? '#eee' : '#8899aa';

            const txt = this.add.text(this.rightX + 8, y, `#${game.id} ${summary}`, {
                fontSize: '11px', fontFamily: 'monospace', color: resultColor,
            }).setInteractive({ useHandCursor: true });

            const gameId = game.id;
            txt.on('pointerdown', () => this.downloadSGFFromDB(gameId));
            txt.on('pointerover', () => txt.setStyle({ color: '#4ecca3' }));
            txt.on('pointerout', () => txt.setStyle({ color: resultColor }));

            this.logEntries.push(txt);
        });
    }

    async downloadSGFFromDB(id) {
        const sgf = await getGameSGF(id);
        if (!sgf) return;

        const blob = new Blob([sgf], { type: 'application/x-go-sgf' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `othello-game-${id}.sgf`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    /* ─── Game Logic ─── */

    startGame() {
        this.time.removeAllEvents();

        if (this.game_state) {
            this.game_state.free();
            this.game_state = null;
        }

        this.boardSize = this.selectedSize;
        this.players = {
            0: this.selectedBlack,
            1: this.selectedWhite,
        };

        const wasm = getWasm();
        this.game_state = new wasm.WasmGame(this.boardSize);
        this.animating = false;
        this.aiFailed = false;

        const s = this.game_state.get_score();
        this.scoreHistory = [{ b: s[0], w: s[1] }];

        this.moveHistory = [];
        this.gameStartTime = new Date();

        this.piecesGroup.clear(true, true);
        this.hintsGroup.clear(true, true);

        if (this.saveBtn) {
            this.saveBtn.destroy();
            this.saveBtn = null;
        }

        this.computeLayout();
        this.drawBoard();
        this.drawGraphBg();
        this.createGameUI();
        this.refresh();
    }

    recordMove(color, row, col) {
        this.moveHistory.push({
            color,
            row, col,
            time: new Date(),
        });
    }

    computeLayout() {
        const W = this.cameras.main.width;
        const H = this.cameras.main.height;
        const gameW = W - PANEL_W - RIGHT_W;
        const gameCx = PANEL_W + gameW / 2;

        const marginTop = 16;
        const belowBoard = 160;
        const availH = H - marginTop - belowBoard;
        const availW = gameW - 60;
        this.cellSize = Math.min(
            Math.floor(availW / this.boardSize),
            Math.floor(availH / this.boardSize),
            MAX_CELL
        );
        this.boardPx = this.cellSize * this.boardSize;
        this.boardX = gameCx - this.boardPx / 2 + 14;
        this.boardY = marginTop;

        const boardBottom = this.boardY + this.boardPx;
        this.statusY = boardBottom + 12;
        this.scoreY = this.statusY + 22;
        this.gameCx = gameCx;

        this.graphW = Math.min(gameW - 80, 400);
        this.graphX = gameCx - this.graphW / 2;
        this.graphY = this.scoreY + 24;
        this.graphH = GRAPH_HEIGHT;
        this.btnY = this.graphY + this.graphH + 16;
    }

    drawBoard() {
        if (this.boardGfx) this.boardGfx.destroy();
        this.boardLabels.forEach(l => l.destroy());
        this.boardLabels = [];

        const g = this.add.graphics();
        this.boardGfx = g;

        g.fillStyle(BOARD_COLOR);
        g.fillRect(this.boardX, this.boardY, this.boardPx, this.boardPx);
        g.lineStyle(1, GRID_COLOR, 0.8);
        for (let i = 0; i <= this.boardSize; i++) {
            const x = this.boardX + i * this.cellSize;
            const y = this.boardY + i * this.cellSize;
            g.lineBetween(x, this.boardY, x, this.boardY + this.boardPx);
            g.lineBetween(this.boardX, y, this.boardX + this.boardPx, y);
        }

        const style = { fontSize: '11px', fontFamily: 'monospace', color: '#667' };
        for (let c = 0; c < this.boardSize; c++) {
            const x = this.boardX + c * this.cellSize + this.cellSize / 2;
            const lbl = this.add.text(x, this.boardY - 12, String.fromCharCode(97 + c), style).setOrigin(0.5);
            this.boardLabels.push(lbl);
        }
        for (let r = 0; r < this.boardSize; r++) {
            const y = this.boardY + r * this.cellSize + this.cellSize / 2;
            const lbl = this.add.text(this.boardX - 12, y, `${r + 1}`, style).setOrigin(0.5);
            this.boardLabels.push(lbl);
        }
    }

    drawGraphBg() {
        if (this.graphBg) this.graphBg.destroy();
        this.graphLabels.forEach(l => l.destroy());
        this.graphLabels = [];

        const g = this.add.graphics();
        this.graphBg = g;

        g.fillStyle(0x16213e, 0.9);
        g.fillRect(this.graphX, this.graphY, this.graphW, this.graphH);
        g.lineStyle(1, 0x2a3a5c);
        g.strokeRect(this.graphX, this.graphY, this.graphW, this.graphH);

        g.lineStyle(1, 0x2a3a5c, 0.5);
        for (let i = 1; i <= 3; i++) {
            const y = this.graphY + this.graphH * (1 - i / 4);
            g.lineBetween(this.graphX, y, this.graphX + this.graphW, y);
        }

        const maxDiscs = this.boardSize * this.boardSize;
        const lbl = { fontSize: '9px', fontFamily: 'monospace', color: '#445' };
        const l1 = this.add.text(this.graphX - 4, this.graphY + this.graphH, '0', lbl).setOrigin(1, 1);
        const l2 = this.add.text(this.graphX - 4, this.graphY + this.graphH / 2,
            `${Math.round(maxDiscs / 2)}`, lbl).setOrigin(1, 0.5);
        const l3 = this.add.text(this.graphX - 4, this.graphY, `${maxDiscs}`, lbl).setOrigin(1, 0);
        this.graphLabels.push(l1, l2, l3);
    }

    createGameUI() {
        if (this.statusText) this.statusText.destroy();
        if (this.scoreText) this.scoreText.destroy();
        if (this.undoBtn) this.undoBtn.destroy();

        this.statusText = this.add.text(this.gameCx, this.statusY, '', {
            fontSize: '16px', fontFamily: 'monospace', color: '#e0e0e0',
        }).setOrigin(0.5);

        this.scoreText = this.add.text(this.gameCx, this.scoreY, '', {
            fontSize: '14px', fontFamily: 'monospace', color: '#aaa',
        }).setOrigin(0.5);

        this.undoBtn = this.add.text(this.gameCx, this.btnY, 'UNDO', {
            fontSize: '14px', fontFamily: 'monospace', color: '#ccc',
            backgroundColor: '#444', padding: { x: 12, y: 5 },
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });
        this.undoBtn.on('pointerdown', () => this.doUndo());
    }

    pushScore() {
        const s = this.game_state.get_score();
        this.scoreHistory.push({ b: s[0], w: s[1] });
    }

    updateGraph() {
        if (this.graphLines) this.graphLines.destroy();
        this.graphLines = this.add.graphics();

        const g = this.graphLines;
        const history = this.scoreHistory;
        const maxDiscs = this.boardSize * this.boardSize;

        const toY = (val) => this.graphY + this.graphH - (val / maxDiscs) * this.graphH;
        const toX = (i) => {
            if (history.length <= 1) return this.graphX + this.graphW / 2;
            return this.graphX + (i / (history.length - 1)) * this.graphW;
        };

        if (history.length === 1) {
            g.fillStyle(0x888888);
            g.fillCircle(toX(0), toY(history[0].b), 3);
            g.fillStyle(0xeeeeee);
            g.fillCircle(toX(0), toY(history[0].w), 3);
            return;
        }

        g.lineStyle(2, 0x888888);
        g.beginPath();
        for (let i = 0; i < history.length; i++) {
            if (i === 0) g.moveTo(toX(i), toY(history[i].b));
            else g.lineTo(toX(i), toY(history[i].b));
        }
        g.strokePath();

        g.lineStyle(2, 0xeeeeee);
        g.beginPath();
        for (let i = 0; i < history.length; i++) {
            if (i === 0) g.moveTo(toX(i), toY(history[i].w));
            else g.lineTo(toX(i), toY(history[i].w));
        }
        g.strokePath();

        const last = history[history.length - 1];
        const lastX = toX(history.length - 1);
        g.fillStyle(0x888888);
        g.fillCircle(lastX, toY(last.b), 4);
        g.fillStyle(0xeeeeee);
        g.fillCircle(lastX, toY(last.w), 4);
    }

    currentPlayerType() {
        return this.players[this.game_state.current_player()];
    }

    async finishGame() {
        const score = this.game_state.get_score();
        let result;
        if (score[0] > score[1]) result = `B+${score[0] - score[1]}`;
        else if (score[1] > score[0]) result = `W+${score[1] - score[0]}`;
        else result = 'Draw';

        const sgf = this.generateSGF();
        await insertGame({
            boardSize: this.boardSize,
            blackPlayer: playerLabel(this.players[0]),
            whitePlayer: playerLabel(this.players[1]),
            result,
            blackScore: score[0],
            whiteScore: score[1],
            sgf,
            playedAt: this.gameStartTime.toISOString(),
        });
        await this.refreshLog();
    }

    refresh() {
        this.piecesGroup.clear(true, true);
        this.hintsGroup.clear(true, true);

        const board = this.game_state.get_board();
        const radius = this.cellSize * 0.38;

        for (let r = 0; r < this.boardSize; r++) {
            for (let c = 0; c < this.boardSize; c++) {
                const cell = board[r * this.boardSize + c];
                if (cell === 0) continue;
                const cx = this.boardX + c * this.cellSize + this.cellSize / 2;
                const cy = this.boardY + r * this.cellSize + this.cellSize / 2;
                const color = cell === 1 ? BLACK_COLOR : WHITE_COLOR;
                const circle = this.add.circle(cx, cy, radius, color);
                this.piecesGroup.add(circle);
            }
        }

        const score = this.game_state.get_score();
        this.scoreText.setText(`Black: ${score[0]}  White: ${score[1]}`);

        this.updateGraph();

        if (this.game_state.is_game_over()) {
            if (score[0] > score[1]) this.statusText.setText('Black wins!');
            else if (score[1] > score[0]) this.statusText.setText('White wins!');
            else this.statusText.setText('Draw!');
            this.finishGame();
            if (this.autoContinue) {
                this.time.delayedCall(1500, () => this.startGame());
            } else {
                this.showSaveButton();
            }
            return;
        }

        const player = this.game_state.current_player() === 0 ? 'Black' : 'White';
        this.statusText.setText(`${player}'s turn`);

        if (this.game_state.must_pass()) {
            const color = this.game_state.current_player() === 0 ? 'B' : 'W';
            this.statusText.setText(`${player} passes`);
            this.time.delayedCall(600, () => {
                this.game_state.pass_turn();
                this.recordMove(color, -1, -1);
                this.pushScore();
                this.refresh();
            });
            return;
        }

        if (this.currentPlayerType() === 'human') {
            this.drawHints();
        } else {
            this.maybeAutoMove();
        }
    }

    drawHints() {
        const moves = this.game_state.get_legal_moves();
        const hintRadius = this.cellSize * 0.15;
        const hintColor = this.game_state.current_player() === 0 ? HINT_BLACK : HINT_WHITE;

        for (let i = 0; i < moves.length; i += 2) {
            const r = moves[i];
            const c = moves[i + 1];
            const cx = this.boardX + c * this.cellSize + this.cellSize / 2;
            const cy = this.boardY + r * this.cellSize + this.cellSize / 2;
            const hint = this.add.circle(cx, cy, hintRadius, hintColor, 0.5);
            hint.setInteractive(
                new Phaser.Geom.Circle(0, 0, this.cellSize / 2),
                Phaser.Geom.Circle.Contains
            );
            hint.on('pointerdown', () => this.humanPlay(r, c));
            hint.on('pointerover', () => hint.setAlpha(0.9));
            hint.on('pointerout', () => hint.setAlpha(0.5));
            this.hintsGroup.add(hint);
        }
    }

    humanPlay(row, col) {
        if (this.animating) return;
        const color = this.game_state.current_player() === 0 ? 'B' : 'W';
        try {
            this.game_state.play_move(row, col);
        } catch (e) {
            console.error('Invalid move:', e);
            return;
        }
        this.recordMove(color, row, col);
        this.pushScore();
        this.refresh();
    }

    maybeAutoMove() {
        const type = this.currentPlayerType();
        if (type === 'human') return;
        if (this.game_state.is_game_over()) return;
        if (this.aiFailed) return;

        const level = parseInt(type.slice(2), 10);
        const color = this.game_state.current_player() === 0 ? 'B' : 'W';

        this.animating = true;
        this.statusText.setText(level === 0 ? 'AI-L0...' : `AI-L${level} thinking...`);
        this.time.delayedCall(300, () => {
            let ok = false;
            try {
                const result = this.game_state.play_ai_move(level);
                if (result.length === 2) {
                    this.recordMove(color, result[0], result[1]);
                } else {
                    this.recordMove(color, -1, -1);
                }
                this.pushScore();
                ok = true;
            } catch (e) {
                console.error('AI error:', e);
                this.aiFailed = true;
                this.statusText.setText('AI error: ' + e);
            }
            this.animating = false;
            if (ok) this.refresh();
        });
    }

    doUndo() {
        if (this.animating) return;
        const hasHuman = this.players[0] === 'human' || this.players[1] === 'human';
        try {
            this.game_state.undo();
            this.scoreHistory.pop();
            this.moveHistory.pop();
            if (hasHuman && this.currentPlayerType() !== 'human') {
                this.game_state.undo();
                this.scoreHistory.pop();
                this.moveHistory.pop();
            }
        } catch (e) {
            // No more undo available
        }
        if (this.saveBtn) {
            this.saveBtn.destroy();
            this.saveBtn = null;
        }
        this.refresh();
    }

    /* ─── SGF Export ─── */

    generateSGF() {
        const score = this.game_state.get_score();
        let result;
        if (score[0] > score[1]) result = `B+${score[0] - score[1]}`;
        else if (score[1] > score[0]) result = `W+${score[1] - score[0]}`;
        else result = 'Draw';

        const dt = this.gameStartTime.toISOString().slice(0, 10);
        const pb = playerLabel(this.players[0]);
        const pw = playerLabel(this.players[1]);

        let sgf = `(;FF[4]GM[2]SZ[${this.boardSize}]\n`;
        sgf += `PB[${pb}]PW[${pw}]\n`;
        sgf += `DT[${dt}]RE[${result}]\n`;

        for (const m of this.moveHistory) {
            const prop = m.color;
            const coord = m.row < 0 ? '' : sgfCoord(m.row, m.col);
            const ts = m.time.toISOString();
            sgf += `;${prop}[${coord}]C[${ts}]\n`;
        }

        sgf += ')\n';
        return sgf;
    }

    showSaveButton() {
        if (this.saveBtn) return;
        this.saveBtn = this.add.text(this.gameCx + 60, this.btnY, 'SAVE', {
            fontSize: '14px', fontFamily: 'monospace', color: '#1a1a2e',
            backgroundColor: '#4ecca3', padding: { x: 12, y: 5 },
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });
        this.saveBtn.on('pointerdown', () => this.downloadSGF());
        this.saveBtn.on('pointerover', () => this.saveBtn.setStyle({ backgroundColor: '#6ee6be' }));
        this.saveBtn.on('pointerout', () => this.saveBtn.setStyle({ backgroundColor: '#4ecca3' }));

        this.undoBtn.setX(this.gameCx - 60);
    }

    downloadSGF() {
        const sgf = this.generateSGF();
        const ts = this.gameStartTime.toISOString().replace(/[:.]/g, '-').slice(0, 19);
        const filename = `othello-${this.boardSize}x${this.boardSize}-${ts}.sgf`;

        const blob = new Blob([sgf], { type: 'application/x-go-sgf' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }
}

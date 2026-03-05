export class MenuScene extends Phaser.Scene {
    constructor() {
        super({ key: 'MenuScene' });
    }

    create() {
        const cx = this.cameras.main.width / 2;

        this.add.text(cx, 60, 'OTHELLO', {
            fontSize: '48px', fontFamily: 'monospace', color: '#e0e0e0',
        }).setOrigin(0.5);

        // Board size
        this.add.text(cx, 150, 'Board Size', {
            fontSize: '22px', fontFamily: 'monospace', color: '#aaa',
        }).setOrigin(0.5);

        const sizes = [4, 6, 8, 10, 12, 14, 16, 18, 20];
        this.selectedSize = 8;
        this.sizeButtons = [];

        const startX = cx - (sizes.length * 40) / 2 + 20;
        sizes.forEach((s, i) => {
            const btn = this.add.text(startX + i * 40, 190, `${s}`, {
                fontSize: '18px', fontFamily: 'monospace', color: '#888',
                backgroundColor: '#333', padding: { x: 6, y: 4 },
            }).setOrigin(0.5).setInteractive({ useHandCursor: true });
            btn.on('pointerdown', () => this.selectSize(s));
            this.sizeButtons.push({ btn, size: s });
        });
        this.updateSizeHighlight();

        // Black player
        this.selectedBlack = 'human';
        this.blackButtons = this.createPlayerSelector(cx, 260, 'Black', 'black');

        // White player
        this.selectedWhite = 'human';
        this.whiteButtons = this.createPlayerSelector(cx, 350, 'White', 'white');

        // Start button
        const startBtn = this.add.text(cx, 470, 'START', {
            fontSize: '28px', fontFamily: 'monospace', color: '#1a1a2e',
            backgroundColor: '#4ecca3', padding: { x: 30, y: 12 },
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });

        startBtn.on('pointerdown', () => {
            this.scene.start('GameScene', {
                size: this.selectedSize,
                black: this.selectedBlack,
                white: this.selectedWhite,
            });
        });
        startBtn.on('pointerover', () => startBtn.setStyle({ backgroundColor: '#6ee6be' }));
        startBtn.on('pointerout', () => startBtn.setStyle({ backgroundColor: '#4ecca3' }));
    }

    createPlayerSelector(cx, y, label, side) {
        this.add.text(cx, y, label, {
            fontSize: '22px', fontFamily: 'monospace', color: '#aaa',
        }).setOrigin(0.5);

        const types = [
            { key: 'human', label: 'Human' },
            { key: 'ai', label: 'AI' },
            { key: 'random', label: 'Random' },
        ];
        const buttons = [];
        const totalW = types.length * 110;
        const sx = cx - totalW / 2 + 55;

        types.forEach((t, i) => {
            const btn = this.add.text(sx + i * 110, y + 36, t.label, {
                fontSize: '18px', fontFamily: 'monospace', color: '#888',
                backgroundColor: '#333', padding: { x: 12, y: 6 },
            }).setOrigin(0.5).setInteractive({ useHandCursor: true });
            btn.on('pointerdown', () => this.selectPlayer(side, t.key));
            buttons.push({ btn, key: t.key });
        });

        this.updatePlayerHighlight(buttons, side === 'black' ? this.selectedBlack : this.selectedWhite);
        return buttons;
    }

    selectSize(s) {
        this.selectedSize = s;
        this.updateSizeHighlight();
    }

    selectPlayer(side, key) {
        if (side === 'black') {
            this.selectedBlack = key;
            this.updatePlayerHighlight(this.blackButtons, key);
        } else {
            this.selectedWhite = key;
            this.updatePlayerHighlight(this.whiteButtons, key);
        }
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
}

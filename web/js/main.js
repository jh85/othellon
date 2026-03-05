import { initWasm } from './OthelloWasm.js';
import { initDB } from './GameDB.js';

async function boot() {
    await Promise.all([initWasm(), initDB()]);

    const config = {
        type: Phaser.AUTO,
        width: 1160,
        height: 700,
        parent: 'game-container',
        backgroundColor: '#1a1a2e',
        scene: [],
    };

    const game = new Phaser.Game(config);

    const { GameScene } = await import('./scenes/GameScene.js');
    game.scene.add('GameScene', GameScene, true);
}

boot().catch(console.error);

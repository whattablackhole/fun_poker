import * as THREE from 'three';

function PokerButton({ x, y }: { x: number, y: number }) {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d')!;
    canvas.width = 300;
    canvas.height = 300;

    ctx.fillStyle = 'yellow';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = 'black';

    const fontSize = Math.min(canvas.width, canvas.height);
    ctx.font = `${fontSize}px Arial`;
    ctx.textAlign = 'center';
    ctx.fillText('D', canvas.width / 2, canvas.height / 1.2);

    const texture = new THREE.CanvasTexture(canvas);


    return (
        <mesh rotation={[Math.PI / 2, Math.PI / 2, 0]} position={[x, y, 0.1]}>
            <cylinderGeometry args={[0.3, 0.3, 0.2, 32]} />
            <meshBasicMaterial key="0" attach="material-0" color={'#8B8000'} />
            <meshBasicMaterial key="1" attach="material-1" map={texture} />
            <meshBasicMaterial key="2" attach="material-2" color={'green'} />
        </mesh>
    );
}

export default PokerButton;
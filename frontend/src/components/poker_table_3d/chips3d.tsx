import * as THREE from 'three';


let config: any[];

function calculateChipSettings(config: any, amount: number) {
    const chips = [];
    for (let i = 0; i < config.length; i++) {
        const cfg = config[i];
        const count = Math.floor(amount / cfg.denomination);
        if (count > 0) {
            chips.push({ texture: cfg.texture, surfaceColor: cfg.surfaceColor, count });
            amount -= count * cfg.denomination;
        }
    }
    return chips;
}

(() => {
    let cfgs = [
        { surfaceColor: 'red', patternColor: "yellow", denomination: 1000 },
        { surfaceColor: 'blue', patternColor: "red", denomination: 500 },
        { surfaceColor: 'green', patternColor: "black", denomination: 100 },
        { surfaceColor: 'orange', patternColor: "black", denomination: 50 },
        { surfaceColor: 'grey', patternColor: "red", denomination: 10 },
        { surfaceColor: 'wheat', patternColor: "red", denomination: 5 },
        { surfaceColor: 'grey', patternColor: "white", denomination: 1 },
    ];

    config = cfgs.map((cfg, index) => {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d')!;
        canvas.width = 300;
        canvas.height = 300;
        drawChip(ctx, cfgs.length - index - 1, cfg.surfaceColor, cfg.patternColor, cfg.denomination);
        const texture = new THREE.CanvasTexture(canvas);

        return { ...cfg, texture };
    })
})();


function drawChip(ctx: CanvasRenderingContext2D, numTriangles: number, surfaceColor: string, patternColor: string, denomination: number, rad: number = 150) {
    const canvasWidth = ctx.canvas.width;
    const canvasHeight = ctx.canvas.height;
    const x = canvasWidth / 2;
    const y = canvasHeight / 2;

    ctx.clearRect(0, 0, canvasWidth, canvasHeight);

    ctx.beginPath();
    ctx.arc(x, y, rad, 0, Math.PI * 2);
    ctx.strokeStyle = patternColor;
    ctx.lineWidth = 5;
    ctx.fillStyle = surfaceColor;
    ctx.fill('evenodd');
    ctx.stroke();
    ctx.closePath();

    let angleStep = (Math.PI * 2) / numTriangles;
    let triangleRad = rad - 20;

    for (let i = 0; i < numTriangles; i++) {
        ctx.beginPath();
        let x1 = x + Math.cos(angleStep * i) * rad;
        let y1 = y + Math.sin(angleStep * i) * rad;
        ctx.moveTo(x1, y1);
        let endX = x + Math.cos(angleStep * i) * triangleRad;
        let endY = y + Math.sin(angleStep * i) * triangleRad;
        ctx.lineTo(endX, endY);
        ctx.strokeStyle = patternColor;
        ctx.lineWidth = 20;
        ctx.stroke();
        ctx.closePath();
    }

    ctx.fillStyle = patternColor;
    ctx.font = `${rad / 2}px Arial`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(denomination.toString(), x, y);
}

function Chips({ x, y, amount }: { x: number, y: number, amount: number }) {
    let chipSettings = calculateChipSettings(config, amount);

    return (
        chipSettings.map((chipSetting, index) => {
            let result = []
            for (let i = chipSetting.count; i > 0; i--) {
                result.push(<mesh rotation={[Math.PI / 2, Math.PI / 2, 0]} position={[x + index * 0.3, y, 0.05 + i * 0.051]}>
                    <cylinderGeometry args={[0.15, 0.15, 0.05, 32]} />
                    <meshBasicMaterial key="0" attach="material-0" color={chipSetting.surfaceColor} />
                    <meshBasicMaterial key="1" attach="material-1" map={chipSetting.texture} />
                    <meshBasicMaterial key="2" attach="material-2" color={"black"} />
                    {/* <Text >

                </Text> */}
                </mesh>);
            }
            return result;

        })


    );
}



export default Chips;
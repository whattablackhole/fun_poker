import { RefObject, useEffect, useRef } from 'react';
import PokerCard from '../poker_card/poker-card.tsx';
import { CardSuit, CardValue } from '../../types/card-types.ts';

function PokerTable() {
    const canvasRef: RefObject<HTMLCanvasElement> = useRef(null);

    let canvas_width = 1000;
    let canvas_height = 1000;
    const centerX = canvas_width / 2;
    const centerY = canvas_height / 2;
    const radius = 250;
    const scaleX = 1.7;
    const scaleY = 1.2;

    useEffect(() => {
        const canvas: HTMLCanvasElement = canvasRef.current as HTMLCanvasElement;

        if (canvas) {
            const ctx = canvas.getContext('2d');

            if (ctx) {
                canvas.width = canvas_width;
                canvas.height = canvas_height;

          
                ctx.scale(scaleX, scaleY);
                ctx.beginPath();
                ctx.arc(centerX / scaleX, centerY / scaleY, radius, 0, Math.PI * 2);
                ctx.strokeStyle = 'green';
                ctx.lineWidth = 10;
                ctx.stroke();
            }
        } else {
            console.error("canvas is null", canvas)
        }
    }, []);
    
    const playerPositions = calculatePlayerPositionsFromCanvas(radius, centerX, centerY, scaleX, scaleY);

    let cards = [
        { cardSuit: CardSuit.Clubs, cardValue: 'A' as CardValue },
        { cardSuit: CardSuit.Diamonds, cardValue: 'Q' as CardValue }
      ];    return (
        <div style={{ position: 'relative' }}>
            {playerPositions.map((position, index) => (
                <div key={index} style={{ position: 'absolute', top: position.y, left: position.x, display: 'flex' }}>
                    {cards.map(({cardValue, cardSuit}) => {
                       return <PokerCard cardSuit={ index === 0 ? cardSuit : CardSuit.Empty} cardValue={ index === 0 ? cardValue: 'Empty' } />
                    })}
                    
                </div>
            ))}
            <canvas ref={canvasRef} />
        </div>
    );
}

function calculatePlayerPositionsFromCanvas(radius: number, xCord: number, yCord: number, scaleX: number, scaleY: number) {
    const numPlayers = 9;
    const positions = [];
    for (let i = 0; i < numPlayers; i++) {
        const angle = ((2 * Math.PI)* i) / numPlayers + Math.PI / 2;
        const cos_ratio = Math.cos(angle);
        const sin_ratio = Math.sin(angle);
        const y_fix = sin_ratio < 0 ? -150 : 0;

        const x = xCord + radius * cos_ratio * scaleX;
        const y = yCord + y_fix + radius * sin_ratio * scaleY;
        positions.push({ x, y });
    }
    return positions;
}

export default PokerTable;
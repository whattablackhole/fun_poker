import { animated, useSpring } from "@react-spring/three";
import { Html } from "@react-three/drei";
import { useEffect } from "react";
import PokerCard from "../poker_card/poker-card";
import { Card, CardPair, Player } from "../../types";

const Card3d = ({ cards, position, index }: { cards: CardPair | undefined, position: { x: number, y: number, z: number }, index: any }) => {
    const { x, y, z } = position;
    const [spring1, api1] = useSpring(() => ({
        position: [0, 0, 0],
        config: { duration: 400 },
    }));

    const [spring2, api2] = useSpring(() => ({
        position: [0, 0, 0],
        config: { duration: 200 },
    }));

    useEffect(() => {
        api1.start({ position: [x, y, z], delay: index * 100 });
        api2.start({ position: [x, y, z], delay: index * 200 });
    });

    return (

        <>
            <animated.mesh position={spring1.position.to((x, y, z) => [x, y, z])}>
                <Html>
                    <div style={{ width: '100px', height: '150px', backgroundColor: 'grey', borderRadius: '10px', padding: '10px', textAlign: 'center' }}>
                        <PokerCard cardSuit={cards?.card1?.suit} cardValue={cards?.card1?.value}></PokerCard>
                    </div>

                </Html>
            </animated.mesh>

            <animated.mesh position={spring2.position.to((x, y, z) => [x, y, z])}>
                <Html>
                    <div style={{ marginLeft: '100px', width: '100px', height: '150px', backgroundColor: 'grey', borderRadius: '10px', padding: '10px', textAlign: 'center' }}>
                        <PokerCard cardSuit={cards?.card2?.suit} cardValue={cards?.card2?.value}></PokerCard>
                    </div>
                </Html>
            </animated.mesh>
        </>
       
    );
};

export default Card3d;
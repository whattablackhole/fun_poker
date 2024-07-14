import { animated, useSpring } from "@react-spring/three";
import { Html } from "@react-three/drei";
import { useEffect } from "react";
import PokerCard from "../poker_card/poker-card";
import { CardPair } from "../../types/card";

const Card3d = ({
  cards,
  position,
  index,
  buttonId,
}: {
  cards: CardPair | undefined;
  position: { x: number; y: number; z: number };
  index: any;
  buttonId?: number;
}) => {
  const { x, y, z } = position;

  const [spring1, api1] = useSpring(
    () => ({
      position: [0, 0, 0],
      config: { duration: 600 },
    }),
    []
  );

  const [spring2, api2] = useSpring(
    () => ({
      position: [0, 0, 0],
      config: { duration: 300 },
    }),
    []
  );

  useEffect(() => {
    api1.start({ position: [0, 0, 0], immediate: true });
    api2.start({ position: [0, 0, 0], immediate: true });
    api1.start({ position: [x, y, z], delay: index * 100 });
    api2.start({ position: [x, y, z], delay: index * 200 });
  }, [buttonId]);
  return (
    <>
      <animated.mesh position={spring1.position.to((x, y, z) => [x, y, z])}>
        {/* TODO: Learn about ranges */}
        <Html zIndexRange={[1, 20]}>
          <div
            style={{
              width: "100%",
              height: "100%",
              marginTop: "15px",
              marginLeft: "15px",
              transform: "rotate(-3deg)",
            }}
          >
            <PokerCard
              cardSuit={cards?.card1?.suit}
              cardValue={cards?.card1?.value}
            ></PokerCard>
          </div>
        </Html>
      </animated.mesh>

      <animated.mesh position={spring2.position.to((x, y, z) => [x, y, z])}>
        <Html zIndexRange={[1, 20]}>
          <div
            style={{
              marginLeft: "100px",
              width: "100%",
              height: "100%",
              marginTop: "17px",
              transform: "rotate(3deg)",
            }}
          >
            <PokerCard
              cardSuit={cards?.card2?.suit}
              cardValue={cards?.card2?.value}
            ></PokerCard>
          </div>
        </Html>
      </animated.mesh>
    </>
  );
};

export default Card3d;

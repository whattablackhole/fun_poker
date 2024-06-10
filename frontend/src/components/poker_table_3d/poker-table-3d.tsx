import { Canvas, useFrame, useThree } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { Html } from "@react-three/drei";
import { TextureLoader, Vector3 } from 'three';
import Card3d from './card3d';
import { Player } from '../../types';
import "./poker-table3d.css";
import { FlagIcon, FlagIconCode } from "react-flag-kit";
import PokerCard from '../poker_card/poker-card';
import PokerButton from './poker-button';
import Chips from './chips3d';
import TimerBanner from '../timer_banner/timer-banner';
import BetHistory from '../../types/bet-history';
import { Street } from '../../types/game_state';

const LogCameraSettings = () => {
  const { camera } = useThree();

  useFrame(() => {
    console.log('Camera position:', camera.position);
    console.log('Camera settings:', {
      near: camera.near,
      far: camera.far,
      projectionMatrix: camera.projectionMatrix,
    });
  });

  return null;
};
const offsetXY = (x: number, y: number, offsetDistance: number): { x: number, y: number } => {
  const distance = Math.sqrt(x * x + y * y);


  const newDistance = Math.max(0, distance - offsetDistance);

  const factor = newDistance / distance;

  const newX = x * factor;
  const newY = y * factor;
  return { x: newX, y: newY }
}


function PokerTable3d({ selfPlayer, players, buttonId, street, betHistory, currPlayerId }: { selfPlayer: Player, betHistory: BetHistory, players: Player[], buttonId: number, currPlayerId: number, street?: Street }) {
  const numberOfCards = 9;
  const radius = 5;

  const playersAndPosition = [];
  // depends on playerblock height and width
  const offsetX = 130 / 100;
  const offsetY = -75 / 100;

  const cardScaleRadiusX = 1.7;
  const cardScaleRadiusY = 1.2;


  for (let i = 0; i < numberOfCards; i++) {
    const angle = (i / numberOfCards) * Math.PI * 2 - Math.PI / 2;
    const x = Math.cos(angle) * cardScaleRadiusX * radius
    const y = Math.sin(angle) * cardScaleRadiusY * radius
    const z = 1;

    playersAndPosition.push({ player: players[i], position: { x, y, z } });
  }
  let buttonPos = playersAndPosition.find((p) => p.player.userId == buttonId)
  let borderTexture = new TextureLoader().load("./src/assets/rubber.avif")
  let deskTexture = new TextureLoader().load("./src/assets/desk-texture.jpg")



  return (

    <Canvas style={{ height: '100vh', width: '100vw', backgroundImage: "url('./src/assets/background.png')", backgroundRepeat: 'no-repeat', backgroundSize: 'cover' }} camera={{
      position: [2.7, -16, 48],
      fov: 15
    }}>

      <ambientLight intensity={0.5} />
      <pointLight position={[10, 10, 10]} castShadow />


      <group>

        <mesh scale={[1.5, 1, 1]}>
          <torusGeometry args={[5, 0.15, 10, 100]} />
          <meshBasicMaterial map={borderTexture} />
        </mesh>

        <PokerButton x={buttonPos?.position.x! - 0.2} y={buttonPos?.position.y! + 0.5}></PokerButton>
        {/* TODO: */}
        <Html position={[-3, 2, 0]} style={{ display: 'flex' }}>
          {street?.cards?.map((card, index) => {
            return <PokerCard cardSuit={card.suit} cardValue={card.value} key={index} />
          })}
          {/* <Card3d cards={player.cards} position={position} key={index} index={index} />
        <Card3d cards={player.cards} position={position} key={index} index={index} />
        <Card3d cards={player.cards} position={position} key={index} index={index} /> */}
        </Html>
        {playersAndPosition.map(({ player, position }, index) => {
          let chipsCords = offsetXY(position.x, position.y, 2);
          let playerBlockCords = { x: position.x - offsetX, y: position.y - offsetY, z: position.z }

          return <>
            <Chips amount={betHistory.getPlayerBetAmount(player.userId, street?.streetStatus)} x={chipsCords.x} y={chipsCords.y} />
            <Card3d cards={player.cards} position={playerBlockCords} index={index} />
            <Html position={new Vector3(playerBlockCords.x, playerBlockCords.y, playerBlockCords.z)}>
              <FlagIcon code={player.country as FlagIconCode} size={34} style={{ position: "absolute", top: "116px" }} />
              <div className="player_info trapezium" style={{ alignSelf: 'center', textAlign: "center" }}>
                <div className="player_info__container">
                  <div className="player_name">
                    {player.userName ?? "NickName"}
                  </div>
                  <div className="divider"></div>
                  <div className="player_bank">
                    {(player.bank ?? "100 000") + " chips"}
                  </div>
                </div>


                {player.userId === currPlayerId ?
                  <TimerBanner timeLeft={100} />

                  : null}
              </div>
            </Html>

          </>
        }


        )}
        <mesh rotation={[Math.PI / 2, 0, 0]} scale={[1.5, 1, 1]}>
          <cylinderGeometry args={[5, 5, 0.1, 100]} />
          <meshBasicMaterial map={deskTexture} />
        </mesh>


      </group>

      <OrbitControls />
      {/* <LogCameraSettings></LogCameraSettings> */}
    </Canvas >

  );
}


export default PokerTable3d;
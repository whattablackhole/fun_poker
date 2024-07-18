import { Canvas, useFrame, useThree, useLoader } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import { Html, Text, Image } from "@react-three/drei";
import { TextureLoader, Vector3 } from "three";
import Card3d from "./card3d";
import "./poker-table3d.css";
import { FlagIcon, FlagIconCode } from "react-flag-kit";
import PokerCard from "../poker_card/poker-card";
import PokerButton from "./poker-button";
import Chips from "./chips3d";
import TimerBanner from "../timer_banner/timer-banner";
import BetHistory from "../../types/bet-history";
import { ActionType, GameStatus, Street } from "../../types/game_state";
import React from "react";
import { Player, PlayerStatus } from "../../types/player";
import * as THREE from "three";

const LogCameraSettings = () => {
  const { camera } = useThree();

  useFrame(() => {
    console.log("Camera position:", camera.position);
    console.log("Camera settings:", {
      near: camera.near,
      far: camera.far,
      projectionMatrix: camera.projectionMatrix,
    });
  });

  return null;
};

const offsetXY = (
  x: number,
  y: number,
  offsetDistance: number
): { x: number; y: number } => {
  const distance = Math.sqrt(x * x + y * y);

  const newDistance = Math.max(0, distance - offsetDistance);

  const factor = newDistance / distance;

  const newX = x * factor;
  const newY = y * factor;
  return { x: newX, y: newY };
};

function PokerTable3d({
  players,
  buttonId,
  street,
  betHistory,
  currPlayerId,
  gameStatus,
}: {
  betHistory: BetHistory;
  players: Player[];
  buttonId?: number;
  currPlayerId?: number;
  street?: Street;
  gameStatus: GameStatus;
}) {
  const radius = 5;

  const playersAndPosition = [];
  // depends on playerblock height and width
  const offsetX = 150 / 100;
  const offsetY = -75 / 100;

  const cardScaleRadiusX = 1.7;
  const cardScaleRadiusY = 1.2;

  for (let i = 0; i < players.length; i++) {
    let angle;
    if (i === 0) {
      angle = -Math.PI / 2;
    } else {
      angle =
        ((players.length - i) / players.length) * Math.PI * 2 - Math.PI / 2;
    }

    const x = Math.cos(angle) * cardScaleRadiusX * radius;
    const y = Math.sin(angle) * cardScaleRadiusY * radius;
    const z = 1;

    playersAndPosition.push({ player: players[i], position: { x, y, z } });
  }

  let buttonPlayer = playersAndPosition.find(
    (p) => p.player.userId == buttonId
  );
  let buttonPos;

  if (buttonPlayer) {
    buttonPos = offsetXY(buttonPlayer.position.x, buttonPlayer.position.y, 1.5);
  }

  // TODO: should be cached, but still maybe need to preload
  const borderTexture = useLoader(TextureLoader, "./src/assets/rubber.avif");
  // const deskTexture = useLoader(TextureLoader, "./src/assets/desk-texture.jpg");

  const material = new THREE.ShaderMaterial({
    vertexShader: `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,
    fragmentShader: `
   varying vec2 vUv;
      void main() {
        vec2 uv = vUv - 0.5;
        float dist = length(uv);
        vec3 color = mix(vec3(48.0 / 255.0, 108.0 / 255.0, 207.0 / 255.0), vec3(0.0, 0.0, 0.0), dist * 1.0);
        gl_FragColor = vec4(color, 1.5);
      }`,
  });

  return (
    <Canvas
      style={{
        height: "100vh",
        width: "100vw",
        backgroundImage: "url('./src/assets/background.jpg')",
        backgroundRepeat: "no-repeat",
        backgroundSize: "cover",
      }}
      camera={{
        position: [0, -36, 38],
        fov: 15,
      }}
      shadows
    >
      <ambientLight intensity={0.5} />
      <pointLight position={[10, 10, 10]} castShadow />

      {gameStatus === GameStatus.WaitingForPlayers ? (
        <Html position={[-1, 2, 2]}>
          <h1 className="waiting_for_players"></h1>
        </Html>
      ) : null}

      <group>
        <mesh scale={[1.5, 1, 1]}>
          <torusGeometry args={[radius, 0.2, 10, 100]} />
          <meshBasicMaterial map={borderTexture} />
        </mesh>

        {buttonPos ? (
          <PokerButton x={buttonPos.x} y={buttonPos.y}></PokerButton>
        ) : null}

        <Chips amount={betHistory.getBankOnPrevStreet()} x={0} y={-0.5} />
        {betHistory.getBankOnPrevStreet() > 0 ? (
          <Html style={{ color: "green" }} position={[0, 0, 0.1]}>
            {betHistory.getBankOnPrevStreet()}
          </Html>
        ) : null}

        <Html position={[-3, 3, 0]} style={{ display: "flex" }}>
          {street?.cards?.map((card, index) => {
            return (
              <PokerCard
                cardSuit={card.suit}
                cardValue={card.value}
                key={index}
              />
            );
          })}
        </Html>
        {playersAndPosition.map(({ player, position }, index) => {
          let chipsCords = offsetXY(position.x, position.y, 2.5);
          let playerBlockCordsOffseted = offsetXY(position.x, position.y, -1);
          let playerBlockCords = {
            x: playerBlockCordsOffseted.x - offsetX,
            y: playerBlockCordsOffseted.y - offsetY + 1.5,
            z: 1,
          };

          return (
            <React.Fragment key={player.userId}>
              <Chips
                amount={betHistory.getPlayerBetAmount(
                  player.userId,
                  street?.streetStatus
                )}
                x={chipsCords.x}
                y={chipsCords.y}
              />
              {betHistory.getPlayerBetAmount(
                player.userId,
                street?.streetStatus
              ) > 0 ? (
                <Html
                  style={{ color: "green" }}
                  position={new Vector3(chipsCords.x, chipsCords.y, 0)}
                >
                  {betHistory.getPlayerBetAmount(
                    player.userId,
                    street?.streetStatus
                  )}
                </Html>
              ) : null}

              <Card3d
                cards={player.cards}
                position={playerBlockCords}
                index={index}
                buttonId={buttonId}
                isVisible={
                  player.status === PlayerStatus.Ready &&
                  player.action?.actionType !== ActionType.Fold &&
                  gameStatus === GameStatus.Active
                }
              />
              <Html
                style={{
                  width: "220px",
                  display: "flex",
                  justifyContent: "center",
                }}
                zIndexRange={[0, 0]}
                position={
                  new Vector3(
                    playerBlockCords.x,
                    playerBlockCords.y,
                    playerBlockCords.z
                  )
                }
              >
                <div
                  style={{
                    width: "170px",
                    height: "170px",
                    backgroundColor: "wheat",
                    opacity: "0.9",
                    display: "flex",
                    justifyContent: "center",
                    alignItems: "center",
                    border: "2px solid black",
                    borderRadius: "20%",
                  }}
                >
                  <img
                    width={150}
                    height={150}
                    src="/src/assets/default_avatar.png"
                  ></img>
                </div>
              </Html>
              <Html
                position={
                  new Vector3(
                    playerBlockCords.x,
                    playerBlockCords.y,
                    playerBlockCords.z
                  )
                }
              >
                {player.country ? (
                  <FlagIcon
                    code={player.country as FlagIconCode}
                    size={34}
                    style={{ position: "absolute", top: "116px" }}
                  />
                ) : null}
                <div
                  className="player_info trapezium"
                  style={{ alignSelf: "center", textAlign: "center" }}
                >
                  <div className="player_info__container">
                    <div className="player_name">
                      {player.userName +
                        (player.status === PlayerStatus.SitOut
                          ? " (Sit Out)"
                          : "") ?? "NickName"}
                    </div>
                    <div className="divider"></div>
                    <div className="player_bank">
                      {(player.bank ?? "100 000") + " chips"}
                    </div>
                  </div>

                  {player.userId === currPlayerId ? (
                    <TimerBanner timeLeft={100} />
                  ) : null}
                </div>
              </Html>
            </React.Fragment>
          );
        })}
        <mesh
          rotation={[Math.PI / 2, 0, 0]}
          scale={[1.5, 1, 1]}
          material={material}
        >
          <cylinderGeometry args={[radius, radius, 0.1, 100]} />
        </mesh>
        <Text
          color="black"
          anchorX="center"
          anchorY="middle"
          position={[0, 0, 0.11]}
          fillOpacity={0.1}
          scale={[1.5, 1, 1]}
        >
          Fun Poker
        </Text>
        <Image
          url="/src/assets/logo_no_background.svg"
          position={[0, 1.5, 0.12]}
          transparent={true}
          zoom={1.5}
          scale={[2, 2]}
          opacity={0.1}
        ></Image>
      </group>

      <OrbitControls
        minAzimuthAngle={0}
        maxAzimuthAngle={0}
        minDistance={30}
        maxDistance={70}
      />
      {/* <LogCameraSettings></LogCameraSettings> */}
    </Canvas>
  );
}

export default PokerTable3d;

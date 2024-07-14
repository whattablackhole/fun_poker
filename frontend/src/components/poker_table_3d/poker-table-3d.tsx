import { Canvas, useFrame, useThree, useLoader } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import { Html } from "@react-three/drei";
import { TextureLoader, Vector3 } from "three";
import Card3d from "./card3d";
import "./poker-table3d.css";
import { FlagIcon, FlagIconCode } from "react-flag-kit";
import PokerCard from "../poker_card/poker-card";
import PokerButton from "./poker-button";
import Chips from "./chips3d";
import TimerBanner from "../timer_banner/timer-banner";
import BetHistory from "../../types/bet-history";
import { GameStatus, Street } from "../../types/game_state";
import React from "react";
import { Player, PlayerStatus } from "../../types/player";

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
  const offsetX = 130 / 100;
  const offsetY = -75 / 100;

  const cardScaleRadiusX = 1.7;
  const cardScaleRadiusY = 1.2;

  for (let i = 0; i < players.length; i++) {
    const angle = (i / players.length) * Math.PI * 2 - Math.PI / 2;
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
  const deskTexture = useLoader(TextureLoader, "./src/assets/desk-texture.jpg");

  return (
    <Canvas
      style={{
        height: "100vh",
        width: "100vw",
        backgroundImage: "url('./src/assets/background.png')",
        backgroundRepeat: "no-repeat",
        backgroundSize: "cover",
      }}
      camera={{
        position: [2.7, -16, 48],
        fov: 15,
      }}
      shadows
    >
      <ambientLight intensity={0.5} />
      <pointLight position={[10, 10, 10]} castShadow />

      {gameStatus === GameStatus.WaitingForPlayers ? (
        <Html position={[-0.5, 1, 1]}>
          <h1 className="waiting_for_players"></h1>
        </Html>
      ) : null}

      <group>
        <mesh scale={[1.5, 1, 1]}>
          <torusGeometry args={[radius, 0.15, 10, 100]} />
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
          let chipsCords = offsetXY(position.x, position.y, 2);
          let playerBlockCordsOffseted = offsetXY(position.x, position.y, -1);
          let playerBlockCords = {
            x: playerBlockCordsOffseted.x - offsetX,
            y: playerBlockCordsOffseted.y - offsetY + 1,
            z: 1,
          };

          return (
            <React.Fragment key={index}>
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

              {player.status === PlayerStatus.Ready &&
              gameStatus === GameStatus.Active ? (
                <Card3d
                  cards={player.cards}
                  position={playerBlockCords}
                  index={index}
                  buttonId={buttonId}
                />
              ) : null}

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
        <mesh rotation={[Math.PI / 2, 0, 0]} scale={[1.5, 1, 1]}>
          <cylinderGeometry args={[radius, radius, 0.1, 100]} />
          <meshBasicMaterial map={deskTexture} />
        </mesh>
      </group>

      <OrbitControls />
      {/* <LogCameraSettings></LogCameraSettings> */}
    </Canvas>
  );
}

export default PokerTable3d;

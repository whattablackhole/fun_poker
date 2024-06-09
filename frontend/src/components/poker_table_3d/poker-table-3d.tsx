import { Canvas, useFrame, useThree } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { Html } from "@react-three/drei";
import { TextureLoader, Vector3 } from 'three';
import Card3d from './card3d';
import { Player } from '../../types';


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


function PokerTable3d({ selfPlayer, players }: { selfPlayer: Player, players: Player[] }) {
  const numberOfCards = 9;
  const radius = 5;

  const playersAndPosition = [];

  const offsetX = 150;
  const offsetY = -75;
  const cardScaleRadiusX = 1.7;
  const cardScaleRadiusY = 1.2;



  for (let i = 0; i < numberOfCards; i++) {
    const angle = (i / numberOfCards) * Math.PI * 2 - Math.PI / 2;
    const x = Math.cos(angle) * cardScaleRadiusX * radius - (offsetX / 100)
    const y = Math.sin(angle) * cardScaleRadiusY * radius - (offsetY / 100)
    const z = 1;
    let currPlayer = players[i];
    let cards;
    if (currPlayer.userId == selfPlayer.userId) {
      cards = selfPlayer.cards;
    }
    playersAndPosition.push({ player: players[i], position: { x, y, z } });
  }

  let borderTexture = new TextureLoader().load("./src/assets/rubber.avif")
  let deskTexture = new TextureLoader().load("./src/assets/desk-texture.jpg")



  return (

    <Canvas style={{ height: '100vh', width: '100vw', backgroundImage: "url('./src/assets/background.png')" }} camera={{
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
        {playersAndPosition.map(({ player, position }, index) => (
          <>
            <Card3d cards={player.cards} position={position} key={index} index={index} />
            <Html position={new Vector3(position.x, position.y, position.z)}>
              <div style={{ alignSelf: 'center', width: "230px", textAlign: "center", backgroundColor: 'wheat', marginTop: '120px' }}>
                <div>
                  {(player.bank ?? "100 000") + " chips"}
                </div>
                <div>
                  {player.userName ?? "NickName"}
                </div>
                {/* {state && players[index]?.userId === state.currPlayerId ?
                  <div>
                    <TimerBanner timeLeft={100} />
                  </div>

                  : null} */}
              </div>
            </Html>

          </>

        ))}
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
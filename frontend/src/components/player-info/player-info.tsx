import { Player, PlayerStatus } from "../../types/player";
import "./player-info.css";

const PlayerInfo = ({
  isWinner,
  player,
}: {
  isWinner: boolean;
  player: Player;
}) => {
  return (
    <div className={`player_info trapezium ${isWinner ? "winner-border" : ""}`}>
      <div className="player_info__container">
        <div className="player_name">
          {player.userName +
            (player.status === PlayerStatus.SitOut ? " (Sit Out)" : "") ??
            "NickName"}
        </div>
        <div className="divider"></div>
        <div className="player_bank">
          {(player.bank ?? "100 000") + " chips"}
        </div>
      </div>
    </div>
  );
};

export default PlayerInfo;

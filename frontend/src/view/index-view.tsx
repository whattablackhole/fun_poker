import NavigationHeader from "../components/navigation_header/navigation-header.tsx";
import LobbiesTable from "../components/navigation_table/lobbies-table.tsx";
import ApiService from "../services/api.service.ts";

function IndexView() {
    const onClickHandler: React.MouseEventHandler<HTMLButtonElement> = () => {
        ApiService.startGame();
    }

    return <div style={{ display: "flex", flexDirection: 'column', gap: "100px" }}>
        <NavigationHeader></NavigationHeader>
        <button onClick={onClickHandler}>Start game</button>
        <LobbiesTable></LobbiesTable>
    </div>

}
export default IndexView;
import NavigationHeader from "../components/navigation_header/navigation-header.tsx";
import LobbiesTable from "../components/navigation_table/lobbies-table.tsx";

function IndexView() {
    return <div style={{ display: "flex", flexDirection: 'column', gap: "100px" }}>
        <NavigationHeader></NavigationHeader>
        <LobbiesTable></LobbiesTable>
    </div>

}
export default IndexView;
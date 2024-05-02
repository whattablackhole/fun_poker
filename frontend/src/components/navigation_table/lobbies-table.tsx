import { useEffect, useState } from "react";
import ApiService from "../../services/api.service";
import { GameName, GameType, LobbyList } from "../../types";

function LobbiesTable() {
    let [lobbyData, setLobbyData] = useState<LobbyList | null>(null)

    useEffect(() => {
        if (!lobbyData) {
            ApiService.getLobbies().then((lobbyList: LobbyList) => {
                setLobbyData(lobbyList);
            })
        }
    }, [lobbyData]);

    const tableDefinition = {
        columns: [{ columnName: 'Name' }, { columnName: 'Author' }, { columnName: 'Game' }, { columnName: 'Type' }, { columnName: 'Players registered' }]
    };

    if (!lobbyData) {
        return <div>Loading...</div>;
    }

    const headCells = tableDefinition.columns.map((element, i) => {
        return <th key={i}>{element.columnName}</th>
    });

    const rowsData = lobbyData.list.map(el => ({
        name: el.name,
        author: el.authorId,
        game: GameName[el.gameName],
        type: GameType[el.gameType],
        registered: el.playersRegistered.toString()
    }))

    const rows = rowsData.map((row, i) => {
        const tds = Object.values(row).map((value, y) => <td key={y}>{value}</td>);

        return <tr key={i} style={{ border: '1px solid black' }}>
            {tds}
        </tr>
    });

    return <div style={{ marginLeft: "5px" }}>
        <table style={{ borderCollapse: "collapse", width: '80vw' }}>
            <thead>
                <tr>{headCells}</tr>
            </thead>
            <tbody>
                {rows}
            </tbody>
        </table>
    </div>
}

export default LobbiesTable;
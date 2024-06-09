import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import GameView from './view/game-view'
import IndexView from './view/index-view'
import PokerTable3d from './components/poker_table_3d/poker-table-3d';
import Game from './components/game/game';

function App() {
  const router = createBrowserRouter([
    {
      path: "/",
      Component: IndexView
    },
    {
      path: "/new-lobby",
      element: <div>Not implemented yet</div>,
    },
    {
      path: "/game",
      Component: GameView
    },
    {
      path: "/table",
      Component: Game
    }
  ]);
  return (
    <RouterProvider router={router} />
  );
}
export default App

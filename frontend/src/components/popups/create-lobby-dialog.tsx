import * as React from 'react';
import clsx from 'clsx';
import { styled, css } from '@mui/system';
import { Modal as BaseModal } from '@mui/base/Modal';
import { Button, FormControl, InputLabel, MenuItem, Select, TextField } from '@mui/material';
import { CreateLobbyRequest } from '../../types/requests';
import { GameName, GameType } from '../../types';
import ApiService from '../../services/api.service';
import { useUser } from '../../App';

export default function CreateLobbyDialog() {
  const [open, setOpen] = React.useState(false);
  const handleOpen = () => setOpen(true);
  const handleClose = () => setOpen(false);
  const [gameName, setGameName] = React.useState('');
  const [gameStructure, setGameStructure] = React.useState('Cash');
  const [gameType, setGameType] = React.useState('Holdem');
  const user = useUser();
  const handleSubmit = () => {
    // const createLobbyData = {
    //   gameName,
    //   gameType,
    //   gameStructure,
    // };

    let request  =  CreateLobbyRequest.create({
      payload: {
        gameName: GameName.Holdem,
        gameType: GameType.Cash,
        name: gameName,
        authorId: user.id
      }
    })
    ApiService.createLobby(request);
    handleClose();
  };
  
  return (
    <div>
      <TriggerButton type="button" onClick={handleOpen}>
        Create Lobby
      </TriggerButton>
      <Modal
        aria-labelledby="unstyled-modal-title"
        aria-describedby="unstyled-modal-description"
        open={open}
        onClose={handleClose}
        slots={{ backdrop: StyledBackdrop }}
      >
        <ModalContent sx={{ width: 700, minHeight: 500 }}>
            <h2>Game Settings:</h2>
            <TextField
              label="Name"
              id="filled-hidden-label-small"
              defaultValue=""
              variant="standard"
              size="small"
              onChange={(e)=>setGameName(e.target.value)}
            />
            <FormControl fullWidth>
              <InputLabel id="game-type-select-label">Game Type</InputLabel>
              <Select
                labelId="game-type-select-label"
                id="game-type-select"
                value={gameType}
                label="Game Type"
              // onChange={handleChange}
              >
                <MenuItem value={'Holdem'}>Holdem</MenuItem>
              </Select>
            </FormControl>
            <FormControl fullWidth>
              <InputLabel id="game-structure-select-label">Game Structure</InputLabel>
              <Select
                labelId="game-structure-select-label"
                id="game-structure-select"
                value={gameStructure}
                label="Game Structure"
              // onChange={handleChange}
              >
                <MenuItem value={'Tournament'}>Tournament</MenuItem>
                <MenuItem value={'Cash'}>Cash</MenuItem>
              </Select>
            </FormControl>


            <Button onClick={()=>handleSubmit()} size='medium' variant='contained' sx={{ width: '200px', alignSelf: 'flex-end', marginTop: 'auto' }}>
              Create
            </Button>
        </ModalContent>
      </Modal>
    </div>
  );
}

const Backdrop = React.forwardRef<
  HTMLDivElement,
  { open?: boolean; className: string }
>((props, ref) => {
  const { open, className, ...other } = props;
  return (
    <div
      className={clsx({ 'base-Backdrop-open': open }, className)}
      ref={ref}
      {...other}
    />
  );
});

const blue = {
  200: '#99CCFF',
  300: '#66B2FF',
  400: '#3399FF',
  500: '#007FFF',
  600: '#0072E5',
  700: '#0066CC',
};

const grey = {
  50: '#F3F6F9',
  100: '#E5EAF2',
  200: '#DAE2ED',
  300: '#C7D0DD',
  400: '#B0B8C4',
  500: '#9DA8B7',
  600: '#6B7A90',
  700: '#434D5B',
  800: '#303740',
  900: '#1C2025',
};

const Modal = styled(BaseModal)`
  position: fixed;
  z-index: 1300;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
`;

const StyledBackdrop = styled(Backdrop)`
  z-index: -1;
  position: fixed;
  inset: 0;
  background-color: rgb(0 0 0 / 0.5);
  -webkit-tap-highlight-color: transparent;
`;

const ModalContent = styled('div')(
  ({ theme }) => css`
    font-family: 'IBM Plex Sans', sans-serif;
    font-weight: 500;
    text-align: start;
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 40px;
    overflow: hidden;
    background-color: ${theme.palette.mode === 'dark' ? grey[900] : '#fff'};
    border-radius: 8px;
    border: 1px solid ${theme.palette.mode === 'dark' ? grey[700] : grey[200]};
    box-shadow: 0 4px 12px
      ${theme.palette.mode === 'dark' ? 'rgb(0 0 0 / 0.5)' : 'rgb(0 0 0 / 0.2)'};
    padding: 24px;
    color: ${theme.palette.mode === 'dark' ? grey[50] : grey[900]};

    & .modal-title {
      margin: 0;
      line-height: 1.5rem;
      margin-bottom: 8px;
    }

    & .modal-description {
      margin: 0;
      line-height: 1.5rem;
      font-weight: 400;
      color: ${theme.palette.mode === 'dark' ? grey[400] : grey[800]};
      margin-bottom: 4px;
    }
  `,
);

const TriggerButton = styled('button')(
  ({ theme }) => css`
    font-family: 'IBM Plex Sans', sans-serif;
    font-weight: 600;
    font-size: 0.875rem;
    line-height: 1.5;
    padding: 8px 16px;
    border-radius: 8px;
    transition: all 150ms ease;
    cursor: pointer;
    background: ${theme.palette.mode === 'dark' ? grey[900] : '#fff'};
    border: 1px solid ${theme.palette.mode === 'dark' ? grey[700] : grey[200]};
    color: ${theme.palette.mode === 'dark' ? grey[200] : grey[900]};
    box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05);

    &:hover {
      background: ${theme.palette.mode === 'dark' ? grey[800] : grey[50]};
      border-color: ${theme.palette.mode === 'dark' ? grey[600] : grey[300]};
    }

    &:active {
      background: ${theme.palette.mode === 'dark' ? grey[700] : grey[100]};
    }

    &:focus-visible {
      box-shadow: 0 0 0 4px ${theme.palette.mode === 'dark' ? blue[300] : blue[200]};
      outline: none;
    }
  `,
);
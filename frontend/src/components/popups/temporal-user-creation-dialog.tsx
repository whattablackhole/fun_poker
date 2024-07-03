import * as React from "react";
import clsx from "clsx";
import { styled, css } from "@mui/system";
import { Modal as BaseModal } from "@mui/base/Modal";
import { Autocomplete, Button, TextField } from "@mui/material";
import { countries } from "../../helpers/countries";
import { useState } from "react";

// TODO: create template for all dialogs with basic style
export default function CreateTempUserDialog({
  submitHandler,
  onCloseHandler,
  open
}: {
  submitHandler: (userName: string, countryCode: string) => any;
  onCloseHandler: () => any,
  open: boolean
}) {
  const [nameError, setNameError] = useState(false);
  const [countryError, setCountryError] = useState(false);
  const [selectedCountry, setSelectedCountry] = useState("");
  const [userName, setUserName] = useState("");

  const validateSelection = () => {
    if (!selectedCountry) {
        setCountryError(true);
    } else {
        setCountryError(false);
    }
  };

  const validateUserName = () => {
    if (userName.length >= 5)  {
        setNameError(false);
    } else {
        setNameError(true);
    }
  }

  const handleCountryChange = (
    event: any,
    newValue: {
      code: string;
      label: string;
    } | null
  ) => {
    if (newValue) {
      setCountryError(false);
      setSelectedCountry(newValue.code);
    }
  };

  const handleUserNameChange = (
    event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ) => {
    setUserName(event.target.value)
    if (event.target.value.length >= 5) {
        setNameError(false);
    }
  }

  return (
    <Modal
      aria-labelledby="unstyled-modal-title"
      aria-describedby="unstyled-modal-description"
      open={open}
      onClose={onCloseHandler}
      slots={{ backdrop: StyledBackdrop }}
    >
      <ModalContent sx={{ width: 700, minHeight: 500 }}>
        <h2>Creation of temporal user :</h2>
        <TextField
          label="Provide user name"
          id="filled-hidden-label-small"
          defaultValue=""
          variant="standard"
          size="small"
          error={nameError}
          helperText={nameError ? 'Please provide name with atleast 5 characters length' : ''}
          onChange={handleUserNameChange}
        />
        <Autocomplete
          options={countries}
          getOptionLabel={(option) => option.label}
          renderInput={(params) => (
            <TextField
              {...params}
              label="Select a country"
              variant="outlined"
              error={countryError}
              helperText={countryError ? 'Please select a country' : ''}
            />
          )}
         
          onChange={handleCountryChange}
          style={{ width: 300 }}
        />

        <Button
          onClick={() => {
            validateSelection();
            validateUserName();
            if (selectedCountry.length == 2 && userName.length >= 5) {
                onCloseHandler();
                submitHandler(userName, selectedCountry);
            }
            
          }}
          size="medium"
          variant="contained"
          sx={{ width: "200px", alignSelf: "flex-end", marginTop: "auto" }}
        >
          Create
        </Button>
      </ModalContent>
    </Modal>
  );
}

const Backdrop = React.forwardRef<
  HTMLDivElement,
  { open?: boolean; className: string }
>((props, ref) => {
  const { open, className, ...other } = props;
  return (
    <div
      className={clsx({ "base-Backdrop-open": open }, className)}
      ref={ref}
      {...other}
    />
  );
});

const blue = {
  200: "#99CCFF",
  300: "#66B2FF",
  400: "#3399FF",
  500: "#007FFF",
  600: "#0072E5",
  700: "#0066CC",
};

const grey = {
  50: "#F3F6F9",
  100: "#E5EAF2",
  200: "#DAE2ED",
  300: "#C7D0DD",
  400: "#B0B8C4",
  500: "#9DA8B7",
  600: "#6B7A90",
  700: "#434D5B",
  800: "#303740",
  900: "#1C2025",
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

const ModalContent = styled("div")(
  ({ theme }) => css`
    font-family: "IBM Plex Sans", sans-serif;
    font-weight: 500;
    text-align: start;
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 40px;
    overflow: hidden;
    background-color: ${theme.palette.mode === "dark" ? grey[900] : "#fff"};
    border-radius: 8px;
    border: 1px solid ${theme.palette.mode === "dark" ? grey[700] : grey[200]};
    box-shadow: 0 4px 12px
      ${theme.palette.mode === "dark" ? "rgb(0 0 0 / 0.5)" : "rgb(0 0 0 / 0.2)"};
    padding: 24px;
    color: ${theme.palette.mode === "dark" ? grey[50] : grey[900]};

    & .modal-title {
      margin: 0;
      line-height: 1.5rem;
      margin-bottom: 8px;
    }

    & .modal-description {
      margin: 0;
      line-height: 1.5rem;
      font-weight: 400;
      color: ${theme.palette.mode === "dark" ? grey[400] : grey[800]};
      margin-bottom: 4px;
    }
  `
);

const TriggerButton = styled("button")(
  ({ theme }) => css`
    font-family: "IBM Plex Sans", sans-serif;
    font-weight: 600;
    font-size: 0.875rem;
    line-height: 1.5;
    padding: 8px 16px;
    border-radius: 8px;
    transition: all 150ms ease;
    cursor: pointer;
    background: ${theme.palette.mode === "dark" ? grey[900] : "#fff"};
    border: 1px solid ${theme.palette.mode === "dark" ? grey[700] : grey[200]};
    color: ${theme.palette.mode === "dark" ? grey[200] : grey[900]};
    box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05);

    &:hover {
      background: ${theme.palette.mode === "dark" ? grey[800] : grey[50]};
      border-color: ${theme.palette.mode === "dark" ? grey[600] : grey[300]};
    }

    &:active {
      background: ${theme.palette.mode === "dark" ? grey[700] : grey[100]};
    }

    &:focus-visible {
      box-shadow: 0 0 0 4px
        ${theme.palette.mode === "dark" ? blue[300] : blue[200]};
      outline: none;
    }
  `
);

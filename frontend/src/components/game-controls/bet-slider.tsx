import * as React from 'react';
import { styled } from '@mui/material/styles';
import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Slider from '@mui/material/Slider';
import MuiInput from '@mui/material/Input';

const Input = styled(MuiInput)`
  width: 42px;
`;


const CustomSlider = styled(Slider)(({ theme }) => ({
  '& .MuiSlider-thumb': {
    width: 24,
    height: 24,
    backgroundRepeat: 'no-repeat',
    backgroundPosition: 'center',
    background: 'linear-gradient(to bottom, #8b0000, red)', // Apply both linear gradient and image as background
    border: '1px solid red',
    '&:hover': {
      boxShadow: '0px 0px 0px 8px rgba(255, 0, 0, 0.16)',
    },
    '&:active': {
      boxShadow: '0px 0px 0px 14px rgba(255, 0, 0, 0.16)',
    },
  },
  '& .MuiSlider-track': {
    height: 8,
    borderRadius: 4,
    backgroundColor: 'red',
    borderColor: 'red'
  },
  '& .MuiSlider-rail': {
    height: 8,
    borderRadius: 4,
    border: '1px solid black',
    backgroundColor: 'grey',
  },
}));

export default function InputSlider() {
  const [value, setValue] = React.useState(30);

  const handleSliderChange = (event: Event, newValue: number | number[]) => {
    setValue(newValue as number);
  };

  const handleInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setValue(event.target.value === '' ? 0 : Number(event.target.value));
  };

  const handleBlur = () => {
    if (value < 0) {
      setValue(0);
    } else if (value > 100) {
      setValue(100);
    }
  };

  return (
    <Box component="div" sx={{ width: 350 }}>
      <Grid container alignItems="center" sx={{
        border: '1px solid #ccc',
        background: 'linear-gradient(to bottom, lightgrey, darkgrey)',
        borderRadius: 1,
        padding: '3px',
        paddingRight: '15px',
        gap: '10px'
      }}>
        <Grid item alignItems="center" display="flex">
          <Input
            value={value}
            size="small"
            onChange={handleInputChange}
            onBlur={handleBlur}
            sx={{
              border: '2px solid #ccc',
              borderRadius: '5px',
              backgroundColor: 'white',
              width: '90px',
            }}
            inputProps={{
              step: 10,
              min: 0,
              max: 100,
              type: 'number',
              'aria-labelledby': 'input-slider',
            }}
          />
        </Grid>
        <Grid item alignItems="center">
        </Grid>
        <Grid item xs alignItems="center" display="flex">
          <CustomSlider
            value={typeof value === 'number' ? value : 0}
            onChange={handleSliderChange}
            aria-labelledby="input-slider"
          />
        </Grid>

      </Grid>
    </Box>
  );
}
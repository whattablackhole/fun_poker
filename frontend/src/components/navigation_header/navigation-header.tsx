import { AppBar } from '@mui/material';
import React from 'react';

interface NavigationHeaderProps {
  children: React.ReactNode;
}

const NavigationHeader: React.FC<NavigationHeaderProps> = ({ children }) => {
  return (
    <AppBar
      position="sticky"
      style={{
        width: '100%',
        height: '90px',
        display: 'flex',
        backgroundColor: 'unset',
        alignItems: 'center',
        justifyContent: 'space-between',
        flexDirection: 'row',
        gap: '10px',
      }}
    >
      {children}
    </AppBar>
  );
};

export default NavigationHeader;
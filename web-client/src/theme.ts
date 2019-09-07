import {colors, createMuiTheme} from '@material-ui/core';

const theme = createMuiTheme({
  palette: {
    primary: {
      main: '#74FAB3',
    },
    secondary: {
      main: '#424242',
    },
    error: {
      main: colors.red['400'],
    },
    background: {
      default: '#fff',
    },
  },
});

export default theme;

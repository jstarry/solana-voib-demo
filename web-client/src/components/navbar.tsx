import React, {FunctionComponent} from 'react';
import {
  AppBar,
  Grid,
  Switch,
  Toolbar,
  Typography,
  makeStyles,
} from '@material-ui/core';

const useStyles = makeStyles({
  root: {
    flexGrow: 1,
  },
  title: {
    flexGrow: 1,
  },
});

type Props = {
  broadcastMode: boolean;
  onModeToggle: () => void;
};

const Navbar: FunctionComponent<Props> = ({broadcastMode, onModeToggle}) => {
  const classes = useStyles({});

  return (
    <div className={classes.root}>
      <AppBar position="static">
        <Toolbar>
          <Typography className={classes.title} variant="h6">
            VoIB Demo
          </Typography>
          <Typography component="div">
            <Grid component="label" container alignItems="center" spacing={1}>
              <Grid item>Consume</Grid>
              <Grid item>
                <Switch checked={broadcastMode} onChange={onModeToggle} />
              </Grid>
              <Grid item>Produce</Grid>
            </Grid>
          </Typography>
        </Toolbar>
      </AppBar>
    </div>
  );
};

export default Navbar;

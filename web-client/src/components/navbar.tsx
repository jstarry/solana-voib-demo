import React, {FunctionComponent} from 'react';
import {AppBar, Grid, Switch, Toolbar, Typography} from '@material-ui/core';
import {makeStyles} from '@material-ui/styles';

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
              <Grid item>Stream</Grid>
              <Grid item>
                <Switch checked={broadcastMode} onChange={onModeToggle} />
              </Grid>
              <Grid item>Broadcast</Grid>
            </Grid>
          </Typography>
        </Toolbar>
      </AppBar>
    </div>
  );
};

export default Navbar;

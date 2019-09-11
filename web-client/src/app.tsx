import React, {useEffect, useState, FunctionComponent} from 'react';
import ReactDOM from 'react-dom';
import {withRouter, HashRouter, RouteComponentProps} from 'react-router-dom';
import {MuiThemeProvider as ThemeProvider} from '@material-ui/core';

import theme from './theme';
import Broadcast from './broadcast';
import Stream from './stream';
import Navbar from './components/navbar';
import WebRtcServer from './webrtc';

const App: FunctionComponent<RouteComponentProps<{}>> = props => {
  const [webRtc, setWebRtc] = useState(null);

  useEffect(() => {
    const webRtc = new WebRtcServer((): void => {
      setWebRtc(webRtc);
    });
  }, []);

  const broadcastMode = props.location.pathname === '/';
  const renderPage = (): any => {
    if (!webRtc) return null;
    return broadcastMode ? (
      <Broadcast webRtc={webRtc} />
    ) : (
      <Stream webRtc={webRtc} />
    );
  };

  const handleToggle = (): void => {
    if (broadcastMode) {
      props.history.replace('/stream');
    } else {
      props.history.replace('/');
    }
  };

  return (
    <>
      <Navbar broadcastMode={broadcastMode} onModeToggle={handleToggle} />
      {renderPage()}
    </>
  );
};

const root = document.getElementById('root');
const AppRouter = withRouter(App);
if (root) {
  ReactDOM.render(
    <HashRouter>
      <ThemeProvider theme={theme}>
        <AppRouter />
      </ThemeProvider>
    </HashRouter>,
    root,
  );
}

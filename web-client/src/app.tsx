import React, {FunctionComponent} from 'react';
import ReactDOM from 'react-dom';
import {withRouter, HashRouter, RouteComponentProps} from 'react-router-dom';
import {ThemeProvider} from '@material-ui/styles';

import theme from './theme';
import Streamer from './streamer';
import Navbar from './components/navbar';

const App: FunctionComponent<RouteComponentProps<{}>> = props => {
  const broadcastMode = props.location.pathname === '/';
  const renderPage = broadcastMode ? <Streamer /> : null;
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
      {renderPage}
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

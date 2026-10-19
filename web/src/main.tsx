import '@fontsource-variable/geist';
import '@fontsource-variable/geist-mono';
import '@radix-ui/themes/tokens/base.css';
import './styles/colors.css';
import '@radix-ui/themes/components.css';
import '@radix-ui/themes/utilities.css';
import './styles/tokens.css';
import './styles/global.css';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { createQueryClient } from './api';
import { App } from './app/App';
import { areas } from './app/areas';
import { legacyPath } from './app/legacy';
import { createAppRouter } from './app/router';
import { readPreference, resolveAppearance } from './shell/theme';
import { watchInputModality } from './ui/inputModality';

const legacy = legacyPath(window.location.hash);
if (legacy) {
  window.history.replaceState(null, '', legacy);
}
watchInputModality();
document.documentElement.dataset.theme = resolveAppearance(
  readPreference(),
  window.matchMedia('(prefers-color-scheme: light)').matches,
);

const root = document.getElementById('root');
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App client={createQueryClient()} router={createAppRouter(areas)} areas={areas} />
    </StrictMode>,
  );
}

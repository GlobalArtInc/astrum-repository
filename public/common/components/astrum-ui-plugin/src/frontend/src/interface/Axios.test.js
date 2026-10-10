
import Axios from 'axios';
import configureAxios from './configureAxios';

global.AS = {
  app: {
    relativePath: '.',
  },
};

describe('Axios', () => {
  it('sends X-Astrum-UI header', () => {
    configureAxios();
    expect(Axios.defaults.headers.common['X-Astrum-UI']).toBe(true);
  });
});

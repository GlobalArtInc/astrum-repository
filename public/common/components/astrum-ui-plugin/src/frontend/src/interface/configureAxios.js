
import axios from 'axios';

/**
 * Configure the axios module's singleton defaults — CSRF headers, baseURL, and
 * a cache-busting _dc param interceptor. Must be called at plugin startup so
 * that subsequent axios calls in the plugin bundle pick up the configuration.
 */
export default function configureAxios() {
  axios.defaults.xsrfCookieName = 'AS-ANTI-CSRF-TOKEN';
  axios.defaults.xsrfHeaderName = 'AS-ANTI-CSRF-TOKEN';
  axios.defaults.baseURL = AS.app.relativePath;
  axios.defaults.headers.common['X-Astrum-UI'] = true;
  axios.interceptors.request.use(config => {
    return {
      ...config,
      params: {
        _dc: new Date().getTime(),
        ...config.params
      }
    };
  });
}

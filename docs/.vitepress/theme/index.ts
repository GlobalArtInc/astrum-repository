import '@fontsource-variable/geist';
import '@fontsource-variable/geist-mono';
import type { Theme } from 'vitepress';
import DefaultTheme from 'vitepress/theme';
import ClientSwitch from './ClientSwitch.vue';
import './custom.css';

const theme: Theme = {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('ClientSwitch', ClientSwitch);
  },
};

export default theme;

import '@fontsource-variable/geist';
import '@fontsource-variable/geist-mono';
import type { Theme } from 'vitepress';
import DefaultTheme from 'vitepress/theme';
import { h } from 'vue';
import ClientSwitch from './ClientSwitch.vue';
import GithubBanner from './GithubBanner.vue';
import './custom.css';

const theme: Theme = {
  extends: DefaultTheme,
  Layout: () => h(DefaultTheme.Layout, null, { 'layout-top': () => h(GithubBanner) }),
  enhanceApp({ app }) {
    app.component('ClientSwitch', ClientSwitch);
  },
};

export default theme;

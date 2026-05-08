
import {inspect} from "@xstate/inspect";

export default function configureDebug() {
  if (window.location.search.startsWith('?debug')) {
    console.log('initialize xstate inspector');
    inspect({
      iframe: false,
      url: 'https://stately.ai/viz?inspect'
    });
  }
}

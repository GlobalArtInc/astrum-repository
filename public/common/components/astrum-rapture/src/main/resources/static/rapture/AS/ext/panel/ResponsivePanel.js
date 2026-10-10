
/*global Ext*/

/**
 * Panel with responsive behavior
 *
 * @since 3.19
 */
Ext.define('AS.ext.panel.ResponsivePanel', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-responsive-panel',

  plugins: {
    responsive: true
  },
  responsiveConfig: {
    'width <= 1366': {
      maxWidth: 600
    },
    'width <= 1600': {
      maxWidth: 800
    },
    'width > 1600': {
      maxWidth: 1000
    }
  }
});

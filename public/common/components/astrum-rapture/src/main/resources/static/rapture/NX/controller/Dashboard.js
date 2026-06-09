
/*global Ext*/

/**
 * Dashboard controller.
 *
 * @since 3.0
 */
Ext.define('NX.controller.Dashboard', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.I18n'
  ],

  views: [
    'dashboard.Welcome'
  ],

  /**
   * @override
   */
  init: function () {
    this.getApplication().getFeaturesController().registerFeature({
      path: '/Welcome',
      mode: 'browse',
      view: 'NX.view.dashboard.Welcome',
      text: NX.I18n.get('Dashboard_Title'),
      iconConfig: {
        file: 'globalart.png',
        variants: ['x16', 'x32']
      },
      weight: 10,
      authenticationRequired: false,
      visible: function() {
        return !NX.State.getValue('astrum.react.welcome', false);
      }
    });
  }

});

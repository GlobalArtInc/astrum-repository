
/*global Ext*/

/**
 * Dashboard controller.
 *
 * @since 3.0
 */
Ext.define('AS.controller.Dashboard', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.I18n'
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
      view: 'AS.view.dashboard.Welcome',
      text: AS.I18n.get('Dashboard_Title'),
      iconConfig: {
        file: 'globalart.png',
        variants: ['x16', 'x32']
      },
      weight: 10,
      authenticationRequired: false,
      visible: function() {
        return !AS.State.getValue('astrum.react.welcome', false);
      }
    });
  }

});

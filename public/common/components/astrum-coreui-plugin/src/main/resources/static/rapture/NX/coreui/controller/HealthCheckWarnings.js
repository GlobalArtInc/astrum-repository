
/*global Ext*/

/**
 * File descriptor warning controller, handles showing messages.
 *
 * @since 3.5
 */
Ext.define('NX.coreui.controller.HealthCheckWarnings', {
  extend: 'NX.app.Controller',

  requires: [
    'NX.I18n',
    'NX.Permissions'
  ],
  refs: [
    {
      ref: 'healthCheckWarnings',
      selector: '#nx-health-check-warnings'
    }
  ],

  /**
   * @override
   */
  init: function() {

    var me = this;

    me.listen({
      controller: {
        '#State': {
          changed: me.stateChanged,
          userAuthenticated: me.stateChanged
        }
      }
    });
  },

  stateChanged: function() {
    var me = this;
    var warningPanel = me.getHealthCheckWarnings();
    var healthChecksFailed = NX.State.getValue('health_checks_failed', false);
    this.user = NX.State.getUser();

    if (warningPanel) {
      if (this.user && this.user.administrator) {
        warningPanel.show();
        if (healthChecksFailed) {
          warningPanel.setIconCls('x-fa fa-exclamation-circle');
          warningPanel.addCls('nx-health-button-red');
          warningPanel.removeCls('nx-health-button-green');
        }
        else {
          warningPanel.setIconCls('x-fa fa-check-circle');
          warningPanel.addCls('nx-health-button-green');
          warningPanel.removeCls('nx-health-button-red');
        }
      } else {
        warningPanel.hide();
      }
    }
  }
});

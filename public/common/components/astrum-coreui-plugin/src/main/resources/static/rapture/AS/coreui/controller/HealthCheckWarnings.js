
/*global Ext*/

/**
 * File descriptor warning controller, handles showing messages.
 *
 * @since 3.5
 */
Ext.define('AS.coreui.controller.HealthCheckWarnings', {
  extend: 'AS.app.Controller',

  requires: [
    'AS.I18n',
    'AS.Permissions'
  ],
  refs: [
    {
      ref: 'healthCheckWarnings',
      selector: '#as-health-check-warnings'
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
    var healthChecksFailed = AS.State.getValue('health_checks_failed', false);
    this.user = AS.State.getUser();

    if (warningPanel) {
      if (this.user && this.user.administrator) {
        warningPanel.show();
        if (healthChecksFailed) {
          warningPanel.setIconCls('x-fa fa-exclamation-circle');
          warningPanel.addCls('as-health-button-red');
          warningPanel.removeCls('as-health-button-green');
        }
        else {
          warningPanel.setIconCls('x-fa fa-check-circle');
          warningPanel.addCls('as-health-button-green');
          warningPanel.removeCls('as-health-button-red');
        }
      } else {
        warningPanel.hide();
      }
    }
  }
});

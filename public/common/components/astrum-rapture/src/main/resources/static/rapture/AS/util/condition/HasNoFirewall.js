
/**
 * A {@link AS.util.condition.Condition} that is satisfied if the Astrum instance does not have Firewall.
 *
 * @since 3.0
 */
Ext.define('AS.util.condition.HasNoFirewall', {
  extend: 'AS.util.condition.Condition',
  requires: [
    'AS.State'
  ],
  api: {
    load: 'AS.direct.healthcheck_Status.hasFirewall',
  },

  bind: function() {
    var me = this,
        controller;
    if (!me.bounded) {
      // Listen for 'clm' state changes to re-evaluate the condition
      controller = AS.getApplication().getController('State');
      me.mon(controller, {
        'clmchanged': me.evaluate,
        scope: me
      });
      me.callParent();
      me.evaluate();
    }
    return me;
  },

  evaluate: function() {
    var me = this;
    if (me.bounded) {
      var clmState = AS.State.getValue("clm", {'enabled': false, 'hasFirewall': false});
      var enabled = clmState['enabled'];
      var hasFirewall = clmState['hasFirewall'];

      // Show Health Check (satisfied) when: IQ is disabled OR IQ doesn't have Firewall
      // Hide Health Check (not satisfied) when: IQ is enabled AND has Firewall
      me.setSatisfied(!(enabled && hasFirewall));
    }
  }
});

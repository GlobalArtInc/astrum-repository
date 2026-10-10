
/*global Ext*/

/**
 * Upgrade Alert.
 *
 */
Ext.define('AS.controller.UpgradeAlert', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.State'
  ],

  refs: [
    { ref: 'alert', selector: 'as-component-upgrade-alert' },
  ],

  /**
   * @override
   */
  init: function () {
    const me = this;

    me.listen({
      component: {
        'as-component-upgrade-alert': {
          beforerender: me.onUpdate
        }
      },
      controller: {
        '#State': {
          userchanged: me.onUpdate,
          changed: me.onUpdate
        },
        '#Permissions': {
          changed: me.onUpdate
        }
      },
    });

    me.callParent();
  },

  /**
   * @private
   */
  onUpdate: function (user) {
    const upgradeAlert = this.getAlert();

    if (!user) {
      upgradeAlert.hide();
    }
    else if (upgradeAlert) {
      upgradeAlert.show();
      upgradeAlert.updateLayout();
    }
  },
});

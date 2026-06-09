
/*global Ext*/

/**
 * Upgrade Alert.
 *
 */
Ext.define('NX.controller.UpgradeAlert', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.State'
  ],

  refs: [
    { ref: 'alert', selector: 'nx-component-upgrade-alert' },
  ],

  /**
   * @override
   */
  init: function () {
    const me = this;

    me.listen({
      component: {
        'nx-component-upgrade-alert': {
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

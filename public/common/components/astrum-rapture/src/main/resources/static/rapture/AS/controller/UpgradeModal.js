
/*global Ext*/

/**
 * Upgrade Modal.
 *
 */
Ext.define('AS.controller.UpgradeModal', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.State'
  ],

  refs: [
    { ref: 'modal', selector: 'as-component-upgrade-modal' },
  ],

  /**
   * @override
   */
  init: function () {
    const me = this;

    me.listen({
      component: {
        'as-component-upgrade-modal': {
          afterrender: me.onUpdate
        }
      },
      controller: {
        '#State': {
          userchanged: me.onUpdate
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
    const upgradeModal = this.getModal();
    if (!user) {
      upgradeModal.hide();
    } else if (upgradeModal) {
      upgradeModal.show();
    }
  }
});

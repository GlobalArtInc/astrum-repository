
/*global Ext*/

/**
 * Upgrade Modal.
 *
 */
Ext.define('NX.controller.UpgradeModal', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.State'
  ],

  refs: [
    { ref: 'modal', selector: 'nx-component-upgrade-modal' },
  ],

  /**
   * @override
   */
  init: function () {
    const me = this;

    me.listen({
      component: {
        'nx-component-upgrade-modal': {
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

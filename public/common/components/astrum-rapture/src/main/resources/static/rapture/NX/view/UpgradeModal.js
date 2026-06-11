
/*global Ext*/

/**
 * Upgrade Modal.
 *
 */
Ext.define('NX.view.UpgradeModal', {
  extend: 'Ext.container.Container',
  alias: 'widget.nx-component-upgrade-modal',
  requires: [
    'NX.view.SecondaryContainer'
  ],

  /**
   * @override
   */
  initComponent: function() {
    const me = this;

    me.items = [
      {
        xtype: 'nx-secondary-container',
        reactView: window.ReactComponents.UpgradeModal,
      }
    ];
    me.show();
    me.callParent();
  },
});

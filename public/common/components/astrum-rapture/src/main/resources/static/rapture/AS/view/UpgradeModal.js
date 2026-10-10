
/*global Ext*/

/**
 * Upgrade Modal.
 *
 */
Ext.define('AS.view.UpgradeModal', {
  extend: 'Ext.container.Container',
  alias: 'widget.as-component-upgrade-modal',
  requires: [
    'AS.view.SecondaryContainer'
  ],

  /**
   * @override
   */
  initComponent: function() {
    const me = this;

    me.items = [
      {
        xtype: 'as-secondary-container',
        reactView: window.ReactComponents.UpgradeModal,
      }
    ];
    me.show();
    me.callParent();
  },
});

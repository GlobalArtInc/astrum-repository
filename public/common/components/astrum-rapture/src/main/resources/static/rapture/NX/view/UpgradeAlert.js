
/*global Ext*/

/**
 * Upgrade Alert.
 *
 */
Ext.define('NX.view.UpgradeAlert', {
  extend: 'Ext.container.Container',
  alias: 'widget.nx-component-upgrade-alert',
  requires: [
    'NX.view.SecondaryContainer'
  ],

  layout: 'fit',

  /**
   * @override
   */
  initComponent: function() {
    const me = this;

    me.items = [
      {
        xtype: 'nx-secondary-container',
        reactView: window.ReactComponents.UpgradeAlert,
        reactViewProps: {
          onClose: function() {
            me.hide();
          }
        }
      }
    ];
    me.show();
    me.callParent();
  },
});


/*global Ext*/

/**
 * Upgrade Alert.
 *
 */
Ext.define('AS.view.UpgradeAlert', {
  extend: 'Ext.container.Container',
  alias: 'widget.as-component-upgrade-alert',
  requires: [
    'AS.view.SecondaryContainer'
  ],

  layout: 'fit',

  /**
   * @override
   */
  initComponent: function() {
    const me = this;

    me.items = [
      {
        xtype: 'as-secondary-container',
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

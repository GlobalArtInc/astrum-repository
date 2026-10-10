
/*global Ext*/

/**
 * CE Hard Limit Banners.
 *
 */
Ext.define('AS.view.CEBanners', {
  extend: 'Ext.container.Container',
  alias: 'widget.as-component-ce-banners',
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
        reactView: window.ReactComponents.CEHardLimitBanners,
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

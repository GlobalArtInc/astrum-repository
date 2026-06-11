
/*global Ext*/

/**
 * CE Hard Limit Banners.
 *
 */
Ext.define('NX.view.CEBanners', {
  extend: 'Ext.container.Container',
  alias: 'widget.nx-component-ce-banners',
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

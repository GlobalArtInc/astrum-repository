
/*global Ext, NX*/

/**
 * Header panel.
 *
 * @since 3.0
 */
Ext.define('NX.view.header.Panel', {
  extend: 'Ext.container.Container',
  alias: 'widget.nx-header-panel',
  requires: [
    'NX.I18n',
    'NX.State'
  ],

  cls: 'nx-header-panel',

  layout: {
    type: 'vbox',
    align: 'stretch',
    pack: 'start'
  },

  ariaRole: 'banner',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {
        xtype: 'toolbar',

        // set height to ensure we have uniform size and not depend on what is in the toolbar
        height: 50,

        anchor: '100%',

        defaults: {
          scale: 'medium'
        },

        items: [
          { xtype: 'nx-header-logo' },
          {
            xtype: 'container',
            layout: {
              type: 'vbox',
              pack: 'center'
            },
            items: [
              {
                xtype: 'label',
                cls: 'productname',
                text: NX.I18n.get('Header_Panel_Logo_Text')
              },
              {
                xtype: 'label',
                cls: 'productspec',
                text: NX.State.getBrandedEditionAndVersion()
              }
            ]
          }
        ]
      }
    ];

    me.callParent();
  }
});

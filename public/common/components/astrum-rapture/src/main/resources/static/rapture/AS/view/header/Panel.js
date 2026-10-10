
/*global Ext, AS*/

/**
 * Header panel.
 *
 * @since 3.0
 */
Ext.define('AS.view.header.Panel', {
  extend: 'Ext.container.Container',
  alias: 'widget.as-header-panel',
  requires: [
    'AS.I18n',
    'AS.State'
  ],

  cls: 'as-header-panel',

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
          { xtype: 'as-header-logo' },
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
                text: AS.I18n.get('Header_Panel_Logo_Text')
              },
              {
                xtype: 'label',
                cls: 'productspec',
                text: AS.State.getBrandedEditionAndVersion()
              }
            ]
          }
        ]
      }
    ];

    me.callParent();
  }
});

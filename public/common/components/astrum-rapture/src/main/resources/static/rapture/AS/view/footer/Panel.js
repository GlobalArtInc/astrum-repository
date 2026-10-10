
/*global Ext*/

/**
 * Footer panel.
 *
 * @since 3.0
 */
Ext.define('AS.view.footer.Panel', {
  extend: 'Ext.container.Container',
  alias: 'widget.as-footer',
  requires: [
    'AS.I18n'
  ],

  cls: 'astrum-footer',
  ariaRole: 'contentinfo',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      layout: {
        type: 'vbox',
        align: 'stretch',
        pack: 'start'
      },

      items: [
        {
          xtype: 'container',
          cls: 'copyright',
          html: AS.I18n.get('Footer_Panel_HTML')
        }
      ]
    });

    this.callParent();
  }
});

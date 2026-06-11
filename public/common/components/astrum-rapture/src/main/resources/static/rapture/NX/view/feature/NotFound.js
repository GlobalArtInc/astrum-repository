
/*global Ext*/

/**
 * Panel shown in case a bookmarked feature is not found (404 like).
 *
 * @since 3.0
 */
Ext.define('NX.view.feature.NotFound', {
  extend: 'Ext.container.Container',
  alias: 'widget.nx-feature-notfound',
  requires: [
    'NX.I18n'
  ],

  cls: [
    'nx-feature-notfound',
    'nx-hr'
  ],

  layout: {
    type: 'vbox',
    align: 'center',
    pack: 'center'
  },

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {
        xtype: 'label',
        cls: 'title',
        text: me.path ? NX.I18n.format('Feature_NotFoundPath_Text', me.path) : NX.I18n.get('Feature_NotFound_Text')
      },
      {
        xtype: 'label',
        cls: 'description',
        // TODO: i18n
        text: 'Sorry the feature you have selected does not exist.  Please make another selection.'
      }
    ];

    me.callParent();
  }

});

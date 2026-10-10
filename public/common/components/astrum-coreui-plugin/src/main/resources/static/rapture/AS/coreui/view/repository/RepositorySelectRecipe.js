
/*global Ext, AS*/

/**
 * Select repository recipe panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.RepositorySelectRecipe', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-repository-selectrecipe',
  requires: [
    'AS.I18n'
  ],

  cls: 'as-hr',

  store: 'RepositoryRecipe',
  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.columns = [
      {
        xtype: 'as-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconName: function() {
          return 'repository-hosted';
        }
      },
      { header: AS.I18n.get('RepositorySelectRecipe_Recipe_Header'), dataIndex: 'name', flex: 1 }
    ];

    me.callParent();
  }

});

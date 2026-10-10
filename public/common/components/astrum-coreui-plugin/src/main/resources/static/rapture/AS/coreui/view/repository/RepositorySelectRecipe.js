
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
  columns: [
    {
      xtype: 'as-iconcolumn',
      width: 36,
      iconVariant: 'x16',
      iconName: function() {
        return 'repository-hosted';
      }
    },
    { header: 'Recipe', dataIndex: 'name', flex: 1 }
  ]

});

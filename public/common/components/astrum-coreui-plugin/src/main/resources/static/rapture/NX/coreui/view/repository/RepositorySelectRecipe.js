
/*global Ext, NX*/

/**
 * Select repository recipe panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.RepositorySelectRecipe', {
  extend: 'NX.view.drilldown.Master',
  alias: 'widget.nx-coreui-repository-selectrecipe',
  requires: [
    'NX.I18n'
  ],

  cls: 'nx-hr',

  store: 'RepositoryRecipe',
  columns: [
    {
      xtype: 'nx-iconcolumn',
      width: 36,
      iconVariant: 'x16',
      iconName: function() {
        return 'repository-hosted';
      }
    },
    { header: 'Recipe', dataIndex: 'name', flex: 1 }
  ]

});

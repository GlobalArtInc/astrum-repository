
/*global Ext, AS*/

/**
 * Select privilege type window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.privilege.PrivilegeSelectType', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-privilege-selecttype',
  requires: [
    'AS.I18n'
  ],

  cls: 'as-hr',

  initComponent: function() {
    var me = this;

    Ext.apply(me, {

      store: 'PrivilegeType',

      columns: [
        {
          xtype: 'as-iconcolumn',
          dataIndex: 'id',
          width: 36,
          iconVariant: 'x16',
          iconNamePrefix: 'privilege-'
        },
        {
          header: AS.I18n.get('Privilege_PrivilegeSelectType_Type_Header'),
          dataIndex: 'name',
          flex: 1
        }
      ]
    });

    me.callParent();
  }

});

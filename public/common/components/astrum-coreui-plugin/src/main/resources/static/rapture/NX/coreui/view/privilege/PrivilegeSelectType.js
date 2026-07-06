
/*global Ext, NX*/

/**
 * Select privilege type window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.privilege.PrivilegeSelectType', {
  extend: 'NX.view.drilldown.Master',
  alias: 'widget.nx-coreui-privilege-selecttype',
  requires: [
    'NX.I18n'
  ],

  cls: 'nx-hr',

  initComponent: function() {
    var me = this;

    Ext.apply(me, {

      store: 'PrivilegeType',

      columns: [
        {
          xtype: 'nx-iconcolumn',
          dataIndex: 'id',
          width: 36,
          iconVariant: 'x16',
          iconNamePrefix: 'privilege-'
        },
        {
          header: NX.I18n.get('Privilege_PrivilegeSelectType_Type_Header'),
          dataIndex: 'name',
          flex: 1
        }
      ]
    });

    me.callParent();
  }

});

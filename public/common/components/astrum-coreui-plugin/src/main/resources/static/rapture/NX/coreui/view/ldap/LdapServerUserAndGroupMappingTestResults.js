
/*global Ext, NX*/

/**
 * LDAP Server User & Group test results window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ldap.LdapServerUserAndGroupMappingTestResults', {
  extend: 'NX.view.ModalDialog',
  alias: 'widget.nx-coreui-ldapserver-userandgroup-testresults',
  requires: [
    'Ext.data.JsonStore',
    'NX.I18n'
  ],

  /**
   * @cfg json array of users (as returned by checking the user mapping)
   */
  config: {
    mappedUsers: undefined
  },

  initComponent: function () {
    var me = this;

    me.title = NX.I18n.get('Ldap_LdapServerUserAndGroupMappingTestResults_Title');
    me.layout = 'fit';
    me.closeable = true;
    me.autoShow = true;
    me.modal = true;
    me.constrain = true;
    me.buttonAlign = 'left';
    me.buttons = [
      { text: NX.I18n.get('Button_Close'), handler: function () {
        this.up('window').close();
      }}
    ];

    me.items = {
      xtype: 'grid',
      columns: [
        { header: NX.I18n.get('Ldap_LdapServerUserAndGroupMappingTestResults_ID_Header'), dataIndex: 'username', flex: 1, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('Ldap_LdapServerUserAndGroupMappingTestResults_Name_Header'), dataIndex: 'realName', flex: 1, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('Ldap_LdapServerUserAndGroupMappingTestResults_Email_Header'), dataIndex: 'email', width: 250, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('Ldap_LdapServerUserAndGroupMappingTestResults_Roles_Header'), dataIndex: 'membership', flex: 3, renderer: Ext.htmlEncode }
      ],
      store: Ext.create('Ext.data.JsonStore', {
        fields: ['username', 'realName', 'email', 'membership'],
        data: me.getMappedUsers()
      })
    };

    me.width = NX.view.ModalDialog.LARGE_MODAL;
    me.maxHeight = me.height = Ext.getBody().getViewSize().height - 100;

    me.callParent();
    me.center();
  }

});

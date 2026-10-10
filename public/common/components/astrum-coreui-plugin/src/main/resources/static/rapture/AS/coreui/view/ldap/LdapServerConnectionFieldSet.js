
/*global Ext, AS*/

/**
 * LDAP Server "Connection" field set.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerConnectionFieldSet', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-coreui-ldapserver-connection-fieldset',
  requires: [
    'AS.I18n',
    'AS.ext.form.field.Hostname'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.defaults = {
      xtype: 'textfield',
        allowBlank: false,
        bindGroup: 'connection'
    };

    me.items = [
      {
        xtype: 'hiddenfield',
        name: 'id'
      },
      {
        name: 'name',
        itemId: 'name',
        fieldLabel: AS.I18n.get('LdapServersConnectionFieldSet_Name_FieldLabel'),
        transformRawValue: Ext.htmlDecode
      },
      {
        xtype: 'label',
        text: AS.I18n.get('LdapServersConnectionFieldSet_Address_Text'),
        style: {
          fontWeight: 'bold',
          display: 'block',
          marginTop: '10px',
          marginBottom: '5px'
        }
      },
      {
        xtype: 'label',
        text: AS.I18n.get('LdapServersConnectionFieldSet_Address_HelpText'),
        style: {
          fontSize: '10px',
          display: 'block',
          marginBottom: '1px'
        }
      },
      {
        xtype: 'combo',
        name: 'protocol',
        itemId: 'protocol',
        cls: 'as-float-left',
        blankText: 'Required',
        width: 85,
        emptyText: AS.I18n.get('LdapServersConnectionFieldSet_Protocol_EmptyText'),
        editable: false,
        store: [
          ['ldap', AS.I18n.get('LdapServersConnectionFieldSet_Protocol_PlainItem')],
          ['ldaps', AS.I18n.get('LdapServersConnectionFieldSet_Protocol_SecureItem')]
        ],
        queryMode: 'local',
        listeners: {
          change: function() {
            var protocol = this.up('form').down('#port');
            protocol.fireEvent('change', protocol, protocol.getValue(), protocol.getValue());
          }
        }
      },
      {
        xtype: 'label',
        cls: 'as-float-left as-interstitial-label',
        text: '://'
      },
      {
        xtype: 'as-hostname',
        name: 'host',
        itemId: 'host',
        cls: 'as-float-left',
        blankText: 'Required',
        width: 405,
        emptyText: AS.I18n.get('LdapServersConnectionFieldSet_Host_EmptyText'),
        listeners: {
          change: function() {
            var protocol = this.up('form').down('#port');
            protocol.fireEvent('change', protocol, protocol.getValue(), protocol.getValue());
          }
        }
      },
      {
        xtype: 'label',
        cls: 'as-float-left as-interstitial-label',
        text: ':'
      },
      {
        xtype: 'numberfield',
        name: 'port',
        itemId: 'port',
        cls: 'as-float-left',
        blankText: 'Required',
        width: 75,
        emptyText: AS.I18n.get('LdapServersConnectionFieldSet_Port_EmptyText'),
        minValue: 1,
        maxValue: 65535,
        allowDecimals: false,
        allowExponential: false,
        useTrustStore: function (field) {
          var form = field.up('form');
          if (form.down('#protocol').getValue() === 'ldaps' && form.down('#host').getValue() && field.getValue()) {
            return {
              name: 'useTrustStore',
              value: form.getRecord() ? form.getRecord().getData().useTrustStore : false,
              host: form.down('#host'),
              port: form.down('#port')
            };
          }
          return undefined;
        }
      },
      {
        name: 'searchBase',
        fieldLabel: AS.I18n.get('LdapServersConnectionFieldSet_Base_FieldLabel'),
        helpText: AS.I18n.get('LdapServersConnectionFieldSet_Base_HelpText'),
        cls: 'as-clear-both'
      },
      {
        xtype: 'combo',
        name: 'authScheme',
        fieldLabel: AS.I18n.get('LdapServersConnectionFieldSet_AuthMethod_FieldLabel'),
        emptyText: AS.I18n.get('LdapServersConnectionFieldSet_AuthMethod_EmptyText'),
        editable: false,
        store: [
          ['simple', AS.I18n.get('LdapServersConnectionFieldSet_AuthMethod_SimpleItem')],
          ['none', AS.I18n.get('LdapServersConnectionFieldSet_AuthMethod_AnonymousItem')],
          ['DIGEST-MD5', AS.I18n.get('LdapServersConnectionFieldSet_AuthMethod_DigestItem')],
          ['CRAM-MD5', AS.I18n.get('LdapServersConnectionFieldSet_AuthMethod_CramItem')]
        ],
        queryMode: 'local',
        listeners: {
          change: function (combo, newValue) {
            this.up('panel').showOrHide('authScheme', newValue);
            this.up('panel').showOrHidePassFieldOrPassButton();
          }
        }
      },
      {
        name: 'authRealm',
        itemId: 'authRealm',
        fieldLabel: AS.I18n.get('LdapServersConnectionFieldSet_SaslRealm_FieldLabel'),
        helpText: AS.I18n.get('LdapServersConnectionFieldSet_SaslRealm_HelpText'),
        allowBlank: true,
        authScheme: ['DIGEST-MD5', 'CRAM-MD5']
      },
      {
        name: 'authUsername',
        itemId: 'authUsername',
        fieldLabel: AS.I18n.get('LdapServersConnectionFieldSet_Username_FieldLabel'),
        helpText: AS.I18n.get('LdapServersConnectionFieldSet_Username_HelpText'),
        authScheme: ['simple', 'DIGEST-MD5', 'CRAM-MD5']
      },
      {
        xtype: 'button',
        itemId: 'authPasswordButton',
        text: AS.I18n.get('LdapServersConnectionFieldSet_ChangePasswordItem'),
        authScheme: ['simple', 'DIGEST-MD5', 'CRAM-MD5'],
        iconCls: 'x-fa fa-key',
        action: 'setpassword'
      },
      {
        xtype: 'as-password',
        name: 'authPassword',
        itemId: 'authPassword',
        fieldLabel: AS.I18n.get('LdapServersConnectionFieldSet_Password_FieldLabel'),
        inputType: 'password',
        helpText: AS.I18n.get('LdapServersConnectionFieldSet_Password_HelpText'),
        authScheme: ['simple', 'DIGEST-MD5', 'CRAM-MD5']
      },
      {
        xtype: 'label',
        text: AS.I18n.get('LdapServersConnectionFieldSet_Rules_Text'),
        style: {
          fontWeight: 'bold',
          display: 'block',
          marginTop: '10px',
          marginBottom: '5px'
        }
      },
      {
        xtype: 'label',
        text: AS.I18n.get('LdapServersConnectionFieldSet_Rules_HelpText'),
        style: {
          fontSize: '10px',
          display: 'block',
          marginBottom: '1px'
        }
      },
      {
        xtype: 'label',
        cls: 'as-float-left as-interstitial-label',
        text: AS.I18n.get('LdapServersConnectionFieldSet_Rules_Text1')
      },
      {
        xtype: 'numberfield',
        name: 'connectionTimeout',
        cls: 'as-float-left',
        width: 70,
        value: 30
      },
      {
        xtype: 'label',
        cls: 'as-float-left as-interstitial-label',
        text: AS.I18n.get('LdapServersConnectionFieldSet_Rules_Text2')
      },
      {
        xtype: 'numberfield',
        name: 'connectionRetryDelay',
        cls: 'as-float-left',
        width: 70,
        value: 300
      },
      {
        xtype: 'label',
        cls: 'as-float-left as-interstitial-label',
        text: AS.I18n.get('LdapServersConnectionFieldSet_Rules_Text3')
      },
      {
        xtype: 'numberfield',
        name: 'maxIncidentsCount',
        cls: 'as-float-left',
        width: 55,
        value: 3
      },
      {
        xtype: 'label',
        cls: 'as-float-left as-interstitial-label',
        text: AS.I18n.get('LdapServersConnectionFieldSet_Rules_Text4')
      }
    ];

    me.callParent();

    me.showOrHidePassFieldOrPassButton();
    me.showOrHide('authScheme', undefined);
  },

  showOrHidePassFieldOrPassButton: function() {
    const me = this;
    var component;
    if (me.up('as-coreui-ldapserver-connection-form').isAddPage) {
      component = me.query('component[itemId=authPasswordButton]')[0];
    }
    else {
      component = me.query('component[itemId=authPassword]')[0];
    }
    component.disable();
    component.hide();
  },

  /**
   * @private
   * Show & enable or hide and disable components that have attributes that matches the specified value.
   * @param attribute name of attribute
   * @param value to be matched in order to show
   */
  showOrHide: function(attribute, value) {
    var me = this,
        form = me.up('form'),
        components = me.query('component[' + attribute + ']');

    Ext.iterate(components, function(component) {
      if (value && component[attribute].indexOf(value) > -1) {
        component.enable();
        component.show();
      }
      else {
        component.disable();
        component.hide();
      }
    });
    if (form && form.rendered) {
      form.isValid();
    }
  }

});

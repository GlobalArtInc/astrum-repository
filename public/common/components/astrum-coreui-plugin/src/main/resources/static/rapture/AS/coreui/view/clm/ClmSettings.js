
/*global Ext, AS*/

/**
 * CLM settings form.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.clm.ClmSettings', {
  extend: 'AS.view.SettingsPanel',
  alias: 'widget.as-coreui-clm-settings',
  requires: [
    'AS.Conditions',
    'AS.I18n',
    'AS.ext.button.Button'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = {
      xtype: 'as-settingsform',
      settingsFormSuccessMessage: 'IQ Server Settings $action',
      api: {
        load: 'AS.direct.clm_CLM.read',
        submit: 'AS.direct.clm_CLM.update'
      },
      editableCondition: AS.Conditions.isPermitted('astrum:settings:update'),
      editableMarker: AS.I18n.get('Clm_ClmSettings_Permission_Error'),
      items: [
        {
          xtype: 'label',
          html: AS.I18n.get('ClmSettings_Html')
        },
        {
          xtype: 'checkbox',
          name: 'enabled',
          fieldLabel: AS.I18n.get('ClmSettings_Enable_FieldLabel'),
          helpText: AS.I18n.get('ClmSettings_Enable_HelpText')
        },
        {
          xtype: 'as-url',
          name: 'url',
          fieldLabel: AS.I18n.get('ClmSettings_URL_FieldLabel'),
          helpText: AS.I18n.get('ClmSettings_URL_HelpText'),
          emptyText: AS.I18n.get('ClmSettings_URL_EmptyText')
        },
        {
          xtype: 'combo',
          name: 'authenticationType',
          fieldLabel: AS.I18n.get('ClmSettings_AuthenticationType_FieldLabel'),
          editable: false,
          store: [
            ['USER', AS.I18n.get('ClmSettings_AuthenticationType_User')],
            ['PKI', AS.I18n.get('ClmSettings_AuthenticationType_Pki')]
          ],
          listeners: {
            'change': function(combo) {
              var userAuthFields = this.up('form').down('#userAuthFields');

              if(combo.getValue() === 'USER') {
                userAuthFields.show();
                userAuthFields.enable();
              }
              else {
                userAuthFields.hide();
                userAuthFields.disable();
              }
            }
          }
        },
        {
          xtype: 'fieldcontainer',
          itemId: 'userAuthFields',
          hidden: true,
          items: [
            {
              xtype:'textfield',
              name: 'username',
              fieldLabel: AS.I18n.get('ClmSettings_Username_FieldLabel'),
              helpText: AS.I18n.get('ClmSettings_Username_HelpText'),
              emptyText: AS.I18n.get('ClmSettings_Username_EmptyText'),
              allowBlank: false,
              inputAttrTpl: 'autocomplete="new-username"'
            },
            {
              xtype: 'as-password',
              name: 'password',
              fieldLabel: AS.I18n.get('ClmSettings_Password_FieldLabel'),
              helpText: AS.I18n.get('ClmSettings_Password_HelpText'),
              emptyText: AS.I18n.get('ClmSettings_Password_EmptyText'),
              allowBlank: false,
              inputAttrTpl: 'autocomplete="new-password"'
            }
          ]
        },
        {
          xtype: 'numberfield',
          name: 'timeoutSeconds',
          fieldLabel: AS.I18n.get('ClmSettings_ConnectionTimeout_FieldLabel'),
          helpText: AS.I18n.get('ClmSettings_ConnectionTimeout_HelpText'),
          emptyText: AS.I18n.get('ClmSettings_ConnectionTimeout_EmptyText'),
          allowBlank: true,
          allowDecimals: false,
          allowExponential: false,
          minValue: 1,
          maxValue: 3600
        },
        {
          xtype: 'textareafield',
          name: 'properties',
          fieldLabel: AS.I18n.get('ClmSettings_Properties_FieldLabel'),
          helpText: AS.I18n.get('ClmSettings_Properties_HelpText'),
          emptyText: AS.I18n.get('ClmSettings_Properties_EmptyText'),
          allowBlank: true
        },
        {
          xtype: 'checkbox',
          name: 'showLink',
          fieldLabel: AS.I18n.get('ClmSettings_Show_Link_FieldLabel'),
          helpText: AS.I18n.get('ClmSettings_Show_Link_HelpText')
        }
      ]
    };

    me.dockedItems = {
      xtype: 'as-actions',
      dock: 'top',
      items: [
        {
          xtype: 'as-button',
          action: 'open',
          text: AS.I18n.get('Clm_Dashboard_Description'),
          iconCls: 'x-fa fa-external-link-alt'
        }
      ]
    };

    me.callParent();

    me.down('as-settingsform').getDockedItems('toolbar[dock="bottom"]')[0].add({
      xtype: 'button', text: AS.I18n.get('ClmSettings_Properties_Verify_Button'), formBind: true, action: 'verify'
    });
  }

});

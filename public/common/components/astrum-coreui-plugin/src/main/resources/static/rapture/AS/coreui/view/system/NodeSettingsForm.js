
/*global Ext, AS*/

/**
 * Form for showing Node information, allowing admins to set a friendly name.
 *
 * @since 3.6
 */
Ext.define('AS.coreui.view.system.NodeSettingsForm', {
  extend: 'AS.view.SettingsForm',
  alias: 'widget.as-coreui-system-node-settings-form',
  requires: [
    'AS.Conditions',
    'AS.I18n',
    'AS.util.Validator'
  ],

  api: {
    submit: 'AS.direct.proui_Node.update'
  },

  settingsFormSuccessMessage: function(data) {
    return AS.I18n.get('Nodes_NodeSettingsForm_Update_Success') + data['friendlyName'];
  },

  initComponent: function() {
    var me = this;

    me.editableMarker = AS.I18n.get('Nodes_NodeSettingsForm_Update_Error');

    me.editableCondition = me.editableCondition || AS.Conditions.isPermitted('astrum:nodes:update');

    me.items = [
      {
        name: 'nodeIdentity',
        itemId: 'nodeIdentity',
        readOnly: true,
        fieldLabel: AS.I18n.get('Nodes_NodeSettingsForm_ID_FieldLabel'),
        helpText: AS.I18n.get('Nodes_NodeSettingsForm_ID_HelpText')
      },
      {
        name: 'socketAddress',
        itemId: 'socketAddress',
        readOnly: true,
        fieldLabel: AS.I18n.get('Nodes_NodeSettingsForm_SocketAddress_FieldLabel'),
        helpText: AS.I18n.get('Nodes_NodeSettingsForm_SocketAddress_HelpText')
      },
      {
        name: 'friendlyName',
        itemId: 'friendlyName',
        fieldLabel: AS.I18n.get('Nodes_NodeSettingsForm_FriendlyName_FieldLabel'),
        helpText: AS.I18n.get('Nodes_NodeSettingsForm_FriendlyName_HelpText'),
        allowBlank: true,
        vtype: 'as-name'
      }
    ];

    me.callParent();
  }

});

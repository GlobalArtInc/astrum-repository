
/*global Ext, NX*/

/**
 * Form for showing Node information, allowing admins to set a friendly name.
 *
 * @since 3.6
 */
Ext.define('NX.coreui.view.system.NodeSettingsForm', {
  extend: 'NX.view.SettingsForm',
  alias: 'widget.nx-coreui-system-node-settings-form',
  requires: [
    'NX.Conditions',
    'NX.I18n',
    'NX.util.Validator'
  ],

  api: {
    submit: 'NX.direct.proui_Node.update'
  },

  settingsFormSuccessMessage: function(data) {
    return NX.I18n.get('Nodes_NodeSettingsForm_Update_Success') + data['friendlyName'];
  },

  initComponent: function() {
    var me = this;

    me.editableMarker = NX.I18n.get('Nodes_NodeSettingsForm_Update_Error');

    me.editableCondition = me.editableCondition || NX.Conditions.isPermitted('astrum:nodes:update');

    me.items = [
      {
        name: 'nodeIdentity',
        itemId: 'nodeIdentity',
        readOnly: true,
        fieldLabel: NX.I18n.get('Nodes_NodeSettingsForm_ID_FieldLabel'),
        helpText: NX.I18n.get('Nodes_NodeSettingsForm_ID_HelpText')
      },
      {
        name: 'socketAddress',
        itemId: 'socketAddress',
        readOnly: true,
        fieldLabel: NX.I18n.get('Nodes_NodeSettingsForm_SocketAddress_FieldLabel'),
        helpText: NX.I18n.get('Nodes_NodeSettingsForm_SocketAddress_HelpText')
      },
      {
        name: 'friendlyName',
        itemId: 'friendlyName',
        fieldLabel: NX.I18n.get('Nodes_NodeSettingsForm_FriendlyName_FieldLabel'),
        helpText: NX.I18n.get('Nodes_NodeSettingsForm_FriendlyName_HelpText'),
        allowBlank: true,
        vtype: 'nx-name'
      }
    ];

    me.callParent();
  }

});

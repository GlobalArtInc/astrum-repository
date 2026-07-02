
/*global Ext, NX*/

/**
 * Capability "Summary" panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.capability.CapabilitySummary', {
  extend: 'Ext.Panel',
  alias: 'widget.nx-coreui-capability-summary',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],
  scrollable: true,
  cls: 'nx-hr',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.title = NX.I18n.get('Capability_CapabilitySummary_Title');

    me.layout = {
      type: 'vbox',
      align: 'stretch'
    };

    me.items = {
      xtype: 'panel',
      ui: 'nx-inset',
      items: [
        {
          xtype: 'panel',
          ui: 'nx-subsection',
          itemId: 'nx-coreui-capability-summary-subsection',
          frame: true,
          layout: 'column',
          weight: 10,
          items: [
            {
              xtype: 'nx-info',
              columnWidth: 1
            }
          ]
        },
        {
          xtype: 'nx-coreui-capability-status',
          ui: 'nx-subsection',
          frame: true,
          title: NX.I18n.get('Capability_CapabilitySummary_Status_Title'),
          weight: 20
        },
        {
          xtype: 'nx-coreui-capability-about',
          ui: 'nx-subsection',
          frame: true,
          title: NX.I18n.get('Capability_CapabilitySummary_About_Title'),
          weight: 30
        },
        {
          xtype: 'nx-settingsform',
          itemId: 'nx-coreui-capability-notes-subsection',
          title: NX.I18n.get('Capability_CapabilitySummary_Notes_Title'),
          weight: 40,
          api: {
            submit: 'NX.direct.capability_Capability.updateNotes'
          },
          settingsFormSuccessMessage: function (data) {
            var description = NX.I18n.format('Capabilities_Update_Success', data['typeName']);
            if (data['description']) {
              description += ' - ' + data['description'];
            }
            return description;
          },
          editableCondition: NX.Conditions.isPermitted('astrum:capabilities:update'),
          editableMarker: NX.I18n.get('Capabilities_Update_Error'),
          items: [
            {
              xtype: 'hiddenfield',
              name: 'id'
            },
            {
              xtype: 'textarea',
              helpText: NX.I18n.get('Capability_CapabilitySummary_Notes_HelpText'),
              name: 'notes',
              allowBlank: true,
              anchor: '100%',
              transformRawValue: Ext.htmlDecode
            }
          ]
        }
      ]
    };

    me.callParent();
  },

  /**
   * @public
   * Shows capability info.
   * @param {Object} info capability info object
   */
  showInfo: function (info) {
    this.down('nx-info').showInfo(info);
  }
});


/*global Ext, AS*/

/**
 * Capability "Summary" panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.capability.CapabilitySummary', {
  extend: 'Ext.Panel',
  alias: 'widget.as-coreui-capability-summary',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],
  scrollable: true,
  cls: 'as-hr',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.title = AS.I18n.get('Capability_CapabilitySummary_Title');

    me.layout = {
      type: 'vbox',
      align: 'stretch'
    };

    me.items = {
      xtype: 'panel',
      ui: 'as-inset',
      items: [
        {
          xtype: 'panel',
          ui: 'as-subsection',
          itemId: 'as-coreui-capability-summary-subsection',
          frame: true,
          layout: 'column',
          weight: 10,
          items: [
            {
              xtype: 'as-info',
              columnWidth: 1
            }
          ]
        },
        {
          xtype: 'as-coreui-capability-status',
          ui: 'as-subsection',
          frame: true,
          title: AS.I18n.get('Capability_CapabilitySummary_Status_Title'),
          weight: 20
        },
        {
          xtype: 'as-coreui-capability-about',
          ui: 'as-subsection',
          frame: true,
          title: AS.I18n.get('Capability_CapabilitySummary_About_Title'),
          weight: 30
        },
        {
          xtype: 'as-settingsform',
          itemId: 'as-coreui-capability-notes-subsection',
          title: AS.I18n.get('Capability_CapabilitySummary_Notes_Title'),
          weight: 40,
          api: {
            submit: 'AS.direct.capability_Capability.updateNotes'
          },
          settingsFormSuccessMessage: function (data) {
            var description = AS.I18n.format('Capabilities_Update_Success', data['typeName']);
            if (data['description']) {
              description += ' - ' + data['description'];
            }
            return description;
          },
          editableCondition: AS.Conditions.isPermitted('astrum:capabilities:update'),
          editableMarker: AS.I18n.get('Capabilities_Update_Error'),
          items: [
            {
              xtype: 'hiddenfield',
              name: 'id'
            },
            {
              xtype: 'textarea',
              helpText: AS.I18n.get('Capability_CapabilitySummary_Notes_HelpText'),
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
    this.down('as-info').showInfo(info);
  }
});

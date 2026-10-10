
/*global Ext, AS*/

/**
 * Configuration specific to Maven repositories.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.facet.Maven2Facet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-maven2-facet',
  requires: [
    'AS.I18n'
  ],

  defaults: {
    allowBlank: false,
    queryMode: 'local',
    itemCls: 'required-field'
  },

  // Default to ATTACHMENT (safer default - prevents phishing via inline HTML)
  contentDisposition: 'ATTACHMENT',

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'as-form-section',
        title: AS.I18n.get('Repository_Facet_Maven2Facet_Title'),

        items: [
          {
            xtype: 'combo',
            name: 'attributes.maven.versionPolicy',
            itemId: 'versionPolicy',
            fieldLabel: AS.I18n.get('Maven2Facet_VersionPolicy_FieldLabel'),
            helpText: AS.I18n.get('Maven2Facet_VersionPolicy_HelpText'),
            emptyText: AS.I18n.get('Maven2Facet_VersionPolicy_EmptyText'),
            editable: false,
            store: [
              ['RELEASE', AS.I18n.get('Maven2Facet_VersionPolicy_ReleaseItem')],
              ['SNAPSHOT', AS.I18n.get('Maven2Facet_VersionPolicy_SnapshotItem')],
              ['MIXED', AS.I18n.get('Maven2Facet_VersionPolicy_MixedItem')]
            ],
            value: 'RELEASE',
            readOnlyOnUpdate: true
          },
          {
            xtype: 'combo',
            name: 'attributes.maven.layoutPolicy',
            fieldLabel: AS.I18n.get('Repository_Facet_Maven2Facet_LayoutPolicy_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_Maven2Facet_LayoutPolicy_HelpText'),
            emptyText: AS.I18n.get('Repository_Facet_Maven2Facet_LayoutPolicy_EmptyText'),
            editable: false,
            store: [
              ['STRICT', AS.I18n.get('Repository_Facet_Maven2Facet_LayoutPolicy_StrictItem')],
              ['PERMISSIVE', AS.I18n.get('Repository_Facet_Maven2Facet_LayoutPolicy_PermissiveItem')]
            ],
            value: 'STRICT'
          },
          {
            xtype: 'combo',
            name: 'attributes.maven.contentDisposition',
            itemId: 'contentDisposition',
            allowBlank: false,
            fieldLabel: AS.I18n.get('Repository_Facet_Maven2Facet_ContentDisposition_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_Maven2Facet_ContentDisposition_HelpText'),
            editable: false,
            store: [
              ['INLINE', AS.I18n.get('Repository_Facet_Maven2Facet_ContentDisposition_Inline')],
              ['ATTACHMENT', AS.I18n.get('Repository_Facet_Maven2Facet_ContentDisposition_Attachment')]
            ],
            value: me.contentDisposition,
            queryMode: 'local'
          },
          {
            xtype: 'displayfield',
            itemId: 'contentDispositionWarning',
            value: '<span class="x-fa fa-exclamation-triangle"></span> ' +
                AS.I18n.get('Repository_Facet_Maven2Facet_ContentDisposition_Warning'),
            cls: 'as-warning-text',
            hidden: me.contentDisposition !== 'INLINE'
          }
        ]
      }
    ];

    me.callParent();

    me.down('#contentDisposition').on('change', function(combo, newValue) {
      me.down('#contentDispositionWarning').setVisible(newValue === 'INLINE');
    });
  }

});


/*global Ext, NX*/

/**
 * Configuration specific to raw repos to set content-disposition
 *
 * @since 3.25
 */
Ext.define('NX.coreui.view.repository.facet.RawFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-raw-facet',
  requires: [
    'NX.I18n'
  ],

  defaults: {
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
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_Raw_Title'),

        items: [
          {
            xtype: 'combo',
            name: 'attributes.raw.contentDisposition',
            itemId: 'contentDisposition',
            allowBlank: false,
            fieldLabel: NX.I18n.get('Repository_Facet_Raw_ContentDisposition_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_Raw_ContentDisposition_HelpText'),
            editable: false,
            store: [
              ['INLINE', NX.I18n.get('Repository_Facet_Raw_ContentDisposition_Inline')],
              ['ATTACHMENT', NX.I18n.get('Repository_Facet_Raw_ContentDisposition_Attachment')]
            ],
            value: me.contentDisposition,
            queryMode: 'local'
          },
          {
            xtype: 'displayfield',
            itemId: 'contentDispositionWarning',
            value: '<span class="x-fa fa-exclamation-triangle"></span> ' +
                NX.I18n.get('Repository_Facet_Raw_ContentDisposition_Warning'),
            cls: 'nx-warning-text',
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

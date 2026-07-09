
/*global Ext, NX*/

/**
 * File created window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.support.FileCreatedForm', {
  extend: 'Ext.form.Panel',
  alias: 'widget.nx-coreui-support-filecreatedform',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    Ext.apply(me, {
      defaults: {
        anchor: '100%'
      },
      items: [
        {
          xtype: 'panel',
          layout: 'hbox',
          style: {
            marginBottom: '10px'
          },
          items: [
            {xtype: 'component', html: me.fileIcon},
            {
              xtype: 'component', html: me.fileType + ' has been created.' +
                  '<br/>You can reference this file on the filesystem or download the file from your browser.',
              margin: '0 0 0 5'
            }
          ]
        },
        {
          xtype: 'textfield',
          name: 'name',
          fieldLabel: NX.I18n.get('Support_FileCreated_Name_FieldLabel'),
          helpText: me.fileType + ' file name',
          readOnly: true
        },
        {
          xtype: 'textfield',
          name: 'size',
          fieldLabel: NX.I18n.get('Support_FileCreated_Size_FieldLabel'),
          helpText: 'Size of ' + me.fileType + ' file in bytes',
          readOnly: true
        },
        {
          xtype: 'textfield',
          name: 'file',
          fieldLabel: NX.I18n.get('Support_FileCreated_Path_FieldLabel'),
          helpText: me.fileType + ' file location',
          readOnly: true,
          selectOnFocus: true
        },
        {
          xtype: 'hidden',
          name: 'truncated'
        }
      ],

      buttonAlign: 'left',
      buttons: [
        {
          text: NX.I18n.get('Support_FileCreated_Download_Button'),
          action: 'download',
          formBind: true,
          bindToEnter: true,
          ui: 'nx-primary',
          iconCls: 'x-fa fa-lock'
        },
        {
          text: NX.I18n.get('Support_FileCreated_Cancel_Button'),
          handler: function() {
            this.up('window').close();
          }
        }
      ]
    });

    me.callParent();
  }
});

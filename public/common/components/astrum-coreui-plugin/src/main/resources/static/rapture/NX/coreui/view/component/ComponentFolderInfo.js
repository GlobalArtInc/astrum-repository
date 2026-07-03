
/*global Ext, NX*/

/**
 * Folder info panel.
 *
 * @since 3.15
 */
Ext.define('NX.coreui.view.component.ComponentFolderInfo', {
  extend: 'Ext.Panel',
  alias: 'widget.nx-coreui-component-componentfolderinfo',
  requires: [
    'NX.ext.button.Button'
  ],

  autoScroll: true,
  cls: 'nx-coreui-component-componentfolderinfo',

  dockedItems: {
    xtype: 'nx-actions',
    items: [
      {
        xtype: 'nx-button',
        text: NX.I18n.get('FolderInfo_Delete_Button'),
        iconCls: 'x-fa fa-trash',
        action: 'deleteFolder',
        hidden: true
      }
    ]
  },

  setModel: function(folder) {
    this.folderModel = folder;
  }

});

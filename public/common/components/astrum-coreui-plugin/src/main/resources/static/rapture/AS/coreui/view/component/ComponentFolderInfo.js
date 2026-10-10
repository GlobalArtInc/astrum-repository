
/*global Ext, AS*/

/**
 * Folder info panel.
 *
 * @since 3.15
 */
Ext.define('AS.coreui.view.component.ComponentFolderInfo', {
  extend: 'Ext.Panel',
  alias: 'widget.as-coreui-component-componentfolderinfo',
  requires: [
    'AS.ext.button.Button'
  ],

  autoScroll: true,
  cls: 'as-coreui-component-componentfolderinfo',

  dockedItems: {
    xtype: 'as-actions',
    items: [
      {
        xtype: 'as-button',
        text: AS.I18n.get('FolderInfo_Delete_Button'),
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
